use crate::app_state::AppState;
use crate::error::OutboxPatternProcessorError;
use crate::http_gateway::HttpGateway;
use crate::http_notification_service::HttpNotificationService;
use crate::outbox::Outbox;
use crate::outbox_destination::OutboxDestination;
use crate::outbox_group::GroupedOutboxed;
use crate::outbox_repository::OutboxRepository;
use crate::outbox_resources::OutboxProcessorResources;
use crate::sns_notification_service::SnsNotificationService;
use crate::sqs_notification_service::SqsNotificationService;
use cron::Schedule;
use sqlx::types::chrono::Utc;
use std::future::Future;
use std::str::FromStr;
use std::time::Duration;
use tracing::instrument;
use tracing::log::{error, info};

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

    pub async fn init_process(self) -> Result<(), OutboxPatternProcessorError> {
        info!("Starting outbox processor...");

        if let Some(box_signal) = self.signal {
            let mut shutdown_signal = Box::into_pin(box_signal);

            info!("Running outbox processor...");
            loop {
                tokio::select! {
                    result = OutboxProcessor::one_shot_process(&self.resources) => {
                        match result {
                            Ok(processed_len) => {
                                if processed_len == 0 {
                                    tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
                                }
                            }
                            Err(error) => {
                                error!("Outbox processor failed with error: {}", error.to_string());
                                tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
                            }
                        }
                    }
                    _ = &mut shutdown_signal => {
                        break;
                    }
                }
            }
        } else {
            loop {
                let result = OutboxProcessor::one_shot_process(&self.resources).await;
                match result {
                    Ok(processed_len) => {
                        if processed_len == 0 {
                            tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
                        }
                    },
                    Err(error) => {
                        error!("Outbox processor failed with error: {}", error.to_string());
                        tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
                    },
                }
            }
        }

        info!("Outbox processor stopped!");

        Ok(())
    }

    pub async fn init_processed_locked_cleaner(self) -> Result<(), OutboxPatternProcessorError> {
        info!("Starting outbox cleaner processor...");

        if let Some(box_signal) = self.signal {
            let mut shutdown_signal = Box::into_pin(box_signal);

            loop {
                tokio::select! {
                    _ = OutboxProcessor::one_shot_processed_locked_cleaner(&self.resources) => {
                        tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
                    }
                    _ = &mut shutdown_signal => {
                        break;
                    }
                }
            }
        } else {
            loop {
                let result = OutboxProcessor::one_shot_processed_locked_cleaner(&self.resources).await;
                if result.is_err() {
                    let error = result.expect_err("Failed to get expected error");
                    error!("Outbox processor cleaner failed with error: {}", error.to_string());
                }
                tokio::time::sleep(Duration::from_secs(self.resources.outbox_execution_interval_in_seconds.unwrap_or(5))).await;
            }
        }

        info!("Outbox processor cleaner stopped!");

        Ok(())
    }

    fn create_app_state(resources: &OutboxProcessorResources) -> Result<AppState, OutboxPatternProcessorError> {
        Ok(AppState {
            postgres_pool: resources.postgres_pool.clone(),
            sqs_client: resources.sqs_client.clone(),
            sns_client: resources.sns_client.clone(),
            http_gateway: HttpGateway::new(resources.http_timeout_in_millis.unwrap_or(3000))?,
            outbox_query_limit: resources.outbox_query_limit,
            delete_after_process_successfully: resources.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: resources.max_in_flight_interval_in_seconds,
            outbox_failure_limit: resources.outbox_failure_limit,
            scheduled_clear_locked_partition: resources.scheduled_clear_locked_partition,
        })
    }

    #[instrument(skip_all, name = "outbox-pattern-processor-cleaner")]
    pub async fn one_shot_processed_locked_cleaner(resources: &OutboxProcessorResources) -> Result<(), OutboxPatternProcessorError> {
        let app_state = Self::create_app_state(resources)?;

        let mut transaction = app_state.begin_transaction().await?;

        if let Some(outbox_clear_schedule) = OutboxRepository::find_cleaner_schedule(&mut transaction).await? {
            if let Ok(schedule) = Schedule::from_str(&outbox_clear_schedule.cron_expression) {
                if let Some(next_execution) = schedule.after(&outbox_clear_schedule.last_execution).next() {
                    let seconds_until_next_execution = (next_execution - Utc::now()).num_seconds();
                    if seconds_until_next_execution <= 0 {
                        OutboxRepository::clear_processed_locked_partition_key(&mut transaction).await?;
                        OutboxRepository::update_last_cleaner_execution(&mut transaction).await?;
                        app_state.commit_transaction(transaction).await?;
                    }
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all, name = "outbox-pattern-processor")]
    pub async fn one_shot_process(resources: &OutboxProcessorResources) -> Result<usize, OutboxPatternProcessorError> {
        let app_state = Self::create_app_state(resources)?;

        let outboxes = OutboxRepository::list(&app_state).await?;
        let outboxes_len = outboxes.len();

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

        let mut transaction = app_state.begin_transaction().await?;

        if app_state.delete_after_process_successfully.unwrap_or(false) {
            OutboxRepository::delete_processed(&app_state, &mut transaction, &successfully_outboxes).await?;
        } else {
            OutboxRepository::mark_as_processed(&app_state, &mut transaction, &successfully_outboxes).await?;
        }

        if !failure_outbox.is_empty() {
            OutboxRepository::increase_attempts(&app_state, &mut transaction, &failure_outbox).await?;
        }

        app_state.commit_transaction(transaction).await?;

        Ok(outboxes_len)
    }

    #[instrument(skip_all)]
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
