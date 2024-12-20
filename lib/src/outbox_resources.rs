use crate::aws::{SnsClient, SqsClient};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct OutboxProcessorResources {
    pub postgres_pool: Pool<Postgres>,
    pub sqs_client: Option<SqsClient>,
    pub sns_client: Option<SnsClient>,
    pub http_timeout_in_millis: Option<u64>,
    pub outbox_query_limit: Option<u32>,
    pub outbox_execution_interval_in_seconds: Option<u64>,
    pub delete_after_process_successfully: Option<bool>,
    pub max_in_flight_interval_in_seconds: Option<u64>,
    pub outbox_failure_limit: Option<u32>,
    pub scheduled_clear_locked_partition: Option<bool>,
    pub outbox_cleaner_execution_interval_in_seconds: Option<u64>,
    pub delay_for_failure_attempt_in_seconds: Option<u64>,
}

impl OutboxProcessorResources {
    pub fn new(
        postgres_pool: Pool<Postgres>,
        sqs_client: Option<SqsClient>,
        sns_client: Option<SnsClient>,
    ) -> Self {
        Self {
            postgres_pool,
            sqs_client,
            sns_client,
            http_timeout_in_millis: None,
            outbox_query_limit: None,
            outbox_execution_interval_in_seconds: None,
            delete_after_process_successfully: None,
            max_in_flight_interval_in_seconds: None,
            outbox_failure_limit: None,
            scheduled_clear_locked_partition: None,
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: None,
        }
    }

    pub fn with_http_timeout_in_millis(
        self,
        http_timeout: u64,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: Some(http_timeout),
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_outbox_query_limit(
        self,
        outbox_query_limit: u32,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: Some(outbox_query_limit),
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_outbox_execution_interval_in_seconds(
        self,
        outbox_execution_interval_in_seconds: u64,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: Some(outbox_execution_interval_in_seconds),
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_delete_after_process_successfully(
        self,
        delete_after_process_successfully: bool,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: Some(delete_after_process_successfully),
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_max_in_flight_interval_in_seconds(
        self,
        max_in_flight_interval_in_seconds: u64,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: Some(max_in_flight_interval_in_seconds),
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_outbox_failure_limit(
        self,
        outbox_failure_limit: u32,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: Some(outbox_failure_limit),
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_scheduled_clear_locked_partition(
        self,
        scheduled_clear_locked_partition: bool,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: Some(scheduled_clear_locked_partition),
            outbox_cleaner_execution_interval_in_seconds: None,
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_outbox_cleaner_execution_interval_in_seconds(
        self,
        outbox_cleaner_execution_interval_in_seconds: u64,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: Some(outbox_cleaner_execution_interval_in_seconds),
            delay_for_failure_attempt_in_seconds: self.delay_for_failure_attempt_in_seconds,
        }
    }

    pub fn with_delay_for_failure_attempt_in_seconds(
        self,
        delay_for_failure_attempt_in_seconds: u64,
    ) -> Self {
        Self {
            postgres_pool: self.postgres_pool,
            sqs_client: self.sqs_client,
            sns_client: self.sns_client,
            http_timeout_in_millis: self.http_timeout_in_millis,
            outbox_query_limit: self.outbox_query_limit,
            outbox_execution_interval_in_seconds: self.outbox_execution_interval_in_seconds,
            delete_after_process_successfully: self.delete_after_process_successfully,
            max_in_flight_interval_in_seconds: self.max_in_flight_interval_in_seconds,
            outbox_failure_limit: self.outbox_failure_limit,
            scheduled_clear_locked_partition: self.scheduled_clear_locked_partition,
            outbox_cleaner_execution_interval_in_seconds: self.outbox_cleaner_execution_interval_in_seconds,
            delay_for_failure_attempt_in_seconds: Some(delay_for_failure_attempt_in_seconds),
        }
    }
}
