use crate::domain::destination::outbox_destination::OutboxDestination;
use crate::domain::outbox::{GroupedOutboxed, Outbox};
use crate::infra::environment::Environment;
use crate::infra::error::AppError;
use crate::repository::outbox::OutboxRepository;
use crate::service::http_notification_service::HttpNotificationService;
use crate::service::sns_notification_service::SqsNotificationService;
use crate::service::sqs_notification_service::SnsNotificationService;
use crate::state::AppState;

pub struct OutboxProcessor;

impl OutboxProcessor {
    pub async fn run(app_state: &AppState) -> Result<(), AppError> {
        let limit = Environment::i32("OUTBOX_QUERY_LIMIT", 50);

        let outboxes = OutboxRepository::list(app_state, limit).await?;

        let grouped_outboxes = Self::group_by_destination(outboxes.clone());

        let sqs_notification_result = SqsNotificationService::send(app_state, &grouped_outboxes).await?;
        let sns_notification_result = SnsNotificationService::send(app_state, &grouped_outboxes).await?;
        let http_notification_result = HttpNotificationService::send(app_state, &grouped_outboxes).await?;

        let mut failure_outbox = vec![];
        failure_outbox.extend(sqs_notification_result.failed);
        failure_outbox.extend(sns_notification_result.failed);
        failure_outbox.extend(http_notification_result.failed);

        let successfully_outboxes = outboxes
            .into_iter()
            .filter(|it| !failure_outbox.iter().any(|failed| failed.idempotent_key == it.idempotent_key))
            .collect::<Vec<Outbox>>();

        if Environment::boolean("DELETE_PROCESSED", false) {
            OutboxRepository::delete_processed(app_state, &successfully_outboxes).await?;
        } else {
            OutboxRepository::mask_as_processed(app_state, &successfully_outboxes).await?;
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
