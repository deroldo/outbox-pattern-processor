use crate::app_state::AppState;
use crate::aws::{SnsClient, SqsClient};
use crate::error::OutboxPatternProcessorError;
use crate::http_gateway::HttpGateway;
use crate::http_notification_service::HttpNotificationService;
use crate::outbox::Outbox;
use crate::outbox_destination::OutboxDestination;
use crate::outbox_repository::OutboxRepository;
use crate::sns_notification_service::SqsNotificationService;
use crate::sqs_notification_service::SnsNotificationService;
use sqlx::{Pool, Postgres};
use std::future::Future;
use std::time::Duration;
use tracing::log::{error, info};
use crate::outbox_group::GroupedOutboxed;

#[derive(Clone)]
pub struct OutboxProcessorResources {
    pub postgres_pool: Pool<Postgres>,
    pub sqs_client: SqsClient,
    pub sns_client: SnsClient,
    pub http_timeout: Option<u64>,
    pub outbox_query_limit: Option<u32>,
    pub outbox_execution_interval_in_seconds: Option<u64>,
    pub delete_after_process_successfully: Option<bool>,
    pub max_in_flight_interval_in_seconds: Option<u64>,
}

pub struct OutboxProcessor {
    resources: OutboxProcessorResources,
    signal: Option<Box<dyn Future<Output = ()> + Send>>,
}

impl OutboxProcessor {
    pub fn new(resources: OutboxProcessorResources) -> Self {
        Self { resources, signal: None }
    }

    pub fn with_graceful_shutdown(
        &self,
        signal: impl Future<Output = ()> + Send + 'static,
    ) -> Self {
        Self {
            resources: self.resources.clone(),
            signal: Some(Box::new(signal)),
        }
    }

    pub async fn init(self) -> Result<(), OutboxPatternProcessorError> {
        info!("Starting outbox processor...");

        if let Some(box_signal) = self.signal {
            let mut shutdown_signal = Box::into_pin(box_signal);

            info!("Running outbox processor...");
            loop {
                tokio::select! {
                    result = OutboxProcessor::one_shot(&self.resources) => {
                        if let Err(error) = result {
                            error!("Outbox processor failed with error: {}", error.to_string());
                        }
                        tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
                    }
                    _ = &mut shutdown_signal => {
                        break;
                    }
                }
            }
        } else {
            loop {
                tokio::select! {
                    result = OutboxProcessor::one_shot(&self.resources) => {
                        if let Err(error) = result {
                            error!("Outbox processor failed with error: {}", error.to_string());
                        }
                        tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
                    }
                }
            }
        }

        info!("Outbox processor stopped!");

        Ok(())
    }

    pub async fn one_shot(resources: &OutboxProcessorResources) -> Result<(), OutboxPatternProcessorError> {
        let app_state = AppState {
            postgres_pool: resources.postgres_pool.clone(),
            sqs_client: resources.sqs_client.clone(),
            sns_client: resources.sns_client.clone(),
            http_gateway: HttpGateway::new(resources.http_timeout.unwrap_or(3000))?,
            outbox_query_limit: resources.outbox_query_limit,
            delete_after_process_successfully: resources.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: resources.max_in_flight_interval_in_seconds,
        };

        let outboxes = OutboxRepository::list(&app_state, app_state.outbox_query_limit.unwrap_or(50) as i32).await?;

        let grouped_outboxes = Self::group_by_destination(outboxes.clone());

        let sqs_notification_result = SqsNotificationService::send(&app_state, &grouped_outboxes).await?;
        let sns_notification_result = SnsNotificationService::send(&app_state, &grouped_outboxes).await?;
        let http_notification_result = HttpNotificationService::send(&app_state, &grouped_outboxes).await?;

        let mut failure_outbox = vec![];
        failure_outbox.extend(sqs_notification_result.failed);
        failure_outbox.extend(sns_notification_result.failed);
        failure_outbox.extend(http_notification_result.failed);

        let successfully_outboxes = outboxes
            .into_iter()
            .filter(|it| !failure_outbox.iter().any(|failed| failed.idempotent_key == it.idempotent_key))
            .collect::<Vec<Outbox>>();

        if app_state.delete_after_process_successfully.unwrap_or(false) {
            OutboxRepository::delete_processed(&app_state, &successfully_outboxes).await?;
        } else {
            OutboxRepository::mark_as_processed(&app_state, &successfully_outboxes).await?;
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
}
