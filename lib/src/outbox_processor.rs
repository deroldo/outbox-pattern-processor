use crate::domain::destination::outbox_destination::OutboxDestination;
use crate::domain::outbox::{GroupedOutboxed, Outbox};
use crate::infra::aws::{SnsClient, SqsClient};
use crate::infra::environment::Environment;
use crate::infra::error::AppError;
use crate::infra::http_gateway::HttpGateway;
use crate::repository::outbox::OutboxRepository;
use crate::service::http_notification_service::HttpNotificationService;
use crate::service::sns_notification_service::SqsNotificationService;
use crate::service::sqs_notification_service::SnsNotificationService;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use tokio::signal;
use tracing::log::{error, info};
use wg::WaitGroup;

#[derive(Clone)]
pub struct OutboxProcessorResources {
    pub postgres_pool: Pool<Postgres>,
    pub sqs_client: SqsClient,
    pub sns_client: SnsClient,
    pub http_gateway: HttpGateway,
}

pub struct OutboxProcessor;

impl OutboxProcessor {
    pub async fn init(
        outbox_processor_resources: OutboxProcessorResources,
        wait_group: WaitGroup,
    ) {
        info!("Starting outbox processor...");

        info!("Running outbox processor...");
        loop {
            tokio::select! {
                result = OutboxProcessor::run(&outbox_processor_resources) => {
                    if let Err(error) = result {
                        error!("Outbox processor failed with error: {}", error.to_string());
                    }
                    tokio::time::sleep(Duration::from_secs(Environment::u64("OUTBOX_QUERY_DELAY_IN_SECONDS", 5))).await;
                }
                _ = Self::shutdown_signal("Stopping outbox processor...") => {
                    break;
                }
            }
        }

        wait_group.done();

        info!("Outbox processor stopped!");
    }

    pub async fn run(resources: &OutboxProcessorResources) -> Result<(), AppError> {
        let limit = Environment::i32("OUTBOX_QUERY_LIMIT", 50);

        let outboxes = OutboxRepository::list(resources, limit).await?;

        let grouped_outboxes = Self::group_by_destination(outboxes.clone());

        let sqs_notification_result = SqsNotificationService::send(resources, &grouped_outboxes).await?;
        let sns_notification_result = SnsNotificationService::send(resources, &grouped_outboxes).await?;
        let http_notification_result = HttpNotificationService::send(resources, &grouped_outboxes).await?;

        let mut failure_outbox = vec![];
        failure_outbox.extend(sqs_notification_result.failed);
        failure_outbox.extend(sns_notification_result.failed);
        failure_outbox.extend(http_notification_result.failed);

        let successfully_outboxes = outboxes
            .into_iter()
            .filter(|it| !failure_outbox.iter().any(|failed| failed.idempotent_key == it.idempotent_key))
            .collect::<Vec<Outbox>>();

        if Environment::boolean("DELETE_PROCESSED", false) {
            OutboxRepository::delete_processed(resources, &successfully_outboxes).await?;
        } else {
            OutboxRepository::mark_as_processed(resources, &successfully_outboxes).await?;
        }

        Ok(())
    }

    fn group_by_destination(outboxes: Vec<Outbox>) -> GroupedOutboxed {
        let mut grouped_outboxes = GroupedOutboxed::default();

        for outbox in outboxes {
            for destination in outbox.destinations.0.clone() {
                match destination {
                    OutboxDestination::SqsDestination(sqs) => {
                        grouped_outboxes.sqs.entry(sqs.queue_url).or_insert(vec![]).push(outbox.clone());
                    },
                    OutboxDestination::SnsDestination(sns) => {
                        grouped_outboxes.sns.entry(sns.topic_arn).or_insert(vec![]).push(outbox.clone());
                    },
                    OutboxDestination::HttpDestination(_) => {
                        grouped_outboxes.http.push(outbox.clone());
                    },
                }
            }
        }

        grouped_outboxes
    }

    async fn shutdown_signal(message: &str) {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
        };

        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        info!("{message}");
    }
}
