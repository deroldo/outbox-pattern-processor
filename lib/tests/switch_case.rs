mod commons;

#[cfg(test)]
mod test {
    use crate::commons::{DefaultData, HttpGatewayMock, TestContext};
    use outbox_pattern_processor::http_destination::HttpDestination;
    use outbox_pattern_processor::outbox::Outbox;
    use outbox_pattern_processor::outbox_destination::OutboxDestination;
    use outbox_pattern_processor::outbox_processor::OutboxProcessor;
    use outbox_pattern_processor::outbox_repository::OutboxRepository;
    use outbox_pattern_processor::outbox_resources::OutboxProcessorResources;
    use outbox_pattern_processor::sns_destination::SnsDestination;
    use outbox_pattern_processor::sqs_destination::SqsDestination;
    use serde_json::json;
    use serial_test::serial;
    use sqlx::types::chrono::Utc;
    use std::collections::HashMap;
    use std::env;
    use std::time::Duration;
    use test_context::test_context;
    use uuid::Uuid;

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_batch_limit(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources =
            OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone()).with_outbox_query_limit(2);

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_none());

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_all_with_batch_limit(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources =
            OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone()).with_outbox_query_limit(2);

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;
        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_all_one_shot(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let processed_len = OutboxProcessor::one_shot_process(&ctx.resources).await.unwrap();
        assert_eq!(3, processed_len);

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_ignoring_fails(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_http_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());
        assert_eq!(1, stored_outbox_1.attempts);

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_respect_attempts_when_is_less_than_threshold(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_outbox_query_limit(1)
            .with_outbox_failure_limit(2);

        let outbox_1 = DefaultData::create_default_http_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;
        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());
        assert_eq!(2, stored_outbox_1.attempts);

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_none());
        assert_eq!(0, stored_outbox_2.attempts);

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_respect_attempts_when_is_greater_than_threshold(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_outbox_query_limit(1)
            .with_outbox_failure_limit(2);

        let outbox_1 = DefaultData::create_default_http_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;
        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;
        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());
        assert_eq!(2, stored_outbox_1.attempts);

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());
        assert_eq!(1, stored_outbox_2.attempts);

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_partition_with_one_shot(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_none());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_partition_with_one_shot_and_scheduled_lock_delete(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_scheduled_clear_locked_partition(true);

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_none());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        let locks = DefaultData::count_processed_locks(ctx).await;
        assert_eq!(2, locks);

        let locks = DefaultData::count_not_processed_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_partition_concurrent_shots(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        for _ in 0..10 {
            DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        }
        for _ in 0..10 {
            let other_outbox = DefaultData::create_default_http_outbox_success(ctx).await;
            HttpGatewayMock::default_mock(ctx, &other_outbox).await;
        }

        let _ = tokio::join!(OutboxProcessor::one_shot_process(&ctx.resources), OutboxProcessor::one_shot_process(&ctx.resources),);

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(21, stored_outboxes.len());

        let outboxes_processed = DefaultData::find_all_outboxes_processed(ctx).await;
        assert_eq!(11, outboxes_processed.len());

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_partition_with_two_shots(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;
        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        let locks = DefaultData::count_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_partition_with_two_shots_and_scheduled_lock_delete(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_scheduled_clear_locked_partition(true);

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;
        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        let locks = DefaultData::count_processed_locks(ctx).await;
        assert_eq!(3, locks);

        let locks = DefaultData::count_not_processed_locks(ctx).await;
        assert_eq!(0, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_http_put(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox = DefaultData::create_http_outbox_success(ctx, "PUT").await;

        HttpGatewayMock::mock_put(ctx, &outbox).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_http_patch(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox = DefaultData::create_http_outbox_success(ctx, "PATCH").await;

        HttpGatewayMock::mock_patch(ctx, &outbox).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_http_with_headers(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let env_value = "my-env-value";
        env::set_var("X_ENV_HEADER_VALUE", env_value);

        let outbox = DefaultData::create_http_outbox_with_headers(
            ctx,
            HashMap::from([
                ("X-ENV-HEADER".to_string(), "{{X_ENV_HEADER_VALUE}}".to_string()),
                ("X-HTTP-HEADER".to_string(), "my-http-value".to_string()),
            ]),
            HashMap::from([("X-OUTBOX-HEADER".to_string(), "my-outbox-value".to_string())]),
        )
        .await;

        HttpGatewayMock::mock_with_headers(
            ctx,
            &outbox,
            HashMap::from([
                ("X-ENV-HEADER".to_string(), env_value.to_string()),
                ("X-HTTP-HEADER".to_string(), "my-http-value".to_string()),
                ("X-OUTBOX-HEADER".to_string(), "my-outbox-value".to_string()),
            ]),
        )
        .await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_sns(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_sns_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_sns_outbox_success(ctx).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_sqs(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_sqs_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_sqs_outbox_success(ctx).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_for_all_destination_successfully(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox = DefaultData::create_outbox(
            ctx,
            None,
            None,
            vec![
                OutboxDestination::HttpDestination(HttpDestination {
                    url: format!("{}/success", ctx.gateway_uri),
                    headers: None,
                    method: None,
                }),
                OutboxDestination::SqsDestination(SqsDestination { queue_url: ctx.queue_url.clone() }),
                OutboxDestination::SnsDestination(SnsDestination { topic_arn: ctx.topic_arn.clone() }),
            ],
            None,
            None,
            None,
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_ignoring_http_fail(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox = DefaultData::create_outbox(
            ctx,
            None,
            None,
            vec![
                OutboxDestination::HttpDestination(HttpDestination {
                    url: format!("{}/failed", ctx.gateway_uri),
                    headers: None,
                    method: None,
                }),
                OutboxDestination::SqsDestination(SqsDestination { queue_url: ctx.queue_url.clone() }),
                OutboxDestination::SnsDestination(SnsDestination { topic_arn: ctx.topic_arn.clone() }),
            ],
            None,
            None,
            None,
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_none());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_ignoring_sqs_fail(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox = DefaultData::create_outbox(
            ctx,
            None,
            None,
            vec![
                OutboxDestination::HttpDestination(HttpDestination {
                    url: format!("{}/success", ctx.gateway_uri),
                    headers: None,
                    method: None,
                }),
                OutboxDestination::SqsDestination(SqsDestination {
                    queue_url: "https://invalid.queue.com".to_string(),
                }),
                OutboxDestination::SnsDestination(SnsDestination { topic_arn: ctx.topic_arn.clone() }),
            ],
            None,
            None,
            None,
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_none());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_ignoring_sns_fail(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox = DefaultData::create_outbox(
            ctx,
            None,
            None,
            vec![
                OutboxDestination::HttpDestination(HttpDestination {
                    url: format!("{}/success", ctx.gateway_uri),
                    headers: None,
                    method: None,
                }),
                OutboxDestination::SqsDestination(SqsDestination { queue_url: ctx.queue_url.clone() }),
                OutboxDestination::SnsDestination(SnsDestination {
                    topic_arn: "invalid::arn".to_string(),
                }),
            ],
            None,
            None,
            None,
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_none());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_deleting_for_each_one_that_result_is_success(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_delete_after_process_successfully(true);

        let outbox_1 = DefaultData::create_default_http_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox.processed_at.is_none());
        assert_eq!(1, stored_outbox.attempts);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_fail_when_destination_is_sqs_but_client_is_none(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), None, ctx.resources.sns_client.clone());

        let outbox_1 = DefaultData::create_default_sqs_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());
        assert_eq!(1, stored_outbox_1.attempts);

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_fail_when_destination_is_sns_but_client_is_none(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), None);

        let outbox_1 = DefaultData::create_default_sns_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());
        assert_eq!(1, stored_outbox_1.attempts);

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_successfully_when_destination_is_http_and_sqs_and_sns_clients_are_none(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), None, None);

        let outbox = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::one_shot_process(&custom_resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_ignoring_scheduled_processes(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let outbox_1 = DefaultData::create_default_scheduled(ctx, Utc::now() + Duration::from_secs(10)).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_none());
        assert_eq!(0, stored_outbox_1.attempts);

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_when_persisted_by_repository(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let mut transaction = ctx.postgres_pool.begin().await.unwrap();

        let outbox_1 = Outbox::http_post_json(Uuid::now_v7(), &format!("{}/success", ctx.gateway_uri), None, &json!({"foo": "bar"})).delay(Utc::now() + Duration::from_secs(10));
        let stored_outbox_1 = OutboxRepository::insert(&mut transaction, outbox_1).await.unwrap();

        let outbox_2 = Outbox::http_post_json(Uuid::now_v7(), &format!("{}/success", ctx.gateway_uri), None, &json!({"foo": "bar"}));
        let stored_outbox_2 = OutboxRepository::insert(&mut transaction, outbox_2).await.unwrap();

        transaction.commit().await.unwrap();

        HttpGatewayMock::default_mock(ctx, &stored_outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &stored_outbox_2).await;

        let _ = OutboxProcessor::one_shot_process(&ctx.resources).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(2, stored_outboxes.len());

        let updated_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == stored_outbox_1.idempotent_key).unwrap();
        assert!(updated_outbox_1.processed_at.is_none());
        assert_eq!(0, updated_outbox_1.attempts);

        let updated_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == stored_outbox_2.idempotent_key).unwrap();
        assert!(updated_outbox_2.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_clear_outbox_lock(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_scheduled_clear_locked_partition(true);

        DefaultData::create_cleaner_schedule(ctx, "* * * * * *").await;

        DefaultData::create_lock(ctx, true).await;
        DefaultData::create_lock(ctx, false).await;
        DefaultData::create_lock(ctx, true).await;

        let _ = OutboxProcessor::one_shot_processed_locked_cleaner(&custom_resources).await;

        let locks = DefaultData::count_processed_locks(ctx).await;
        assert_eq!(0, locks);

        let locks = DefaultData::count_not_processed_locks(ctx).await;
        assert_eq!(1, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_ignore_clear_outbox_lock_when_resource_is_locked(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_scheduled_clear_locked_partition(true);

        DefaultData::create_cleaner_schedule(ctx, "* * * * * *").await;

        DefaultData::create_lock(ctx, true).await;
        DefaultData::create_lock(ctx, false).await;
        DefaultData::create_lock(ctx, true).await;

        let mut transaction = ctx.postgres_pool.begin().await.unwrap();
        let _ = OutboxRepository::find_cleaner_schedule(&mut transaction).await;

        let _ = OutboxProcessor::one_shot_processed_locked_cleaner(&custom_resources).await;

        let locks = DefaultData::count_processed_locks(ctx).await;
        assert_eq!(2, locks);

        let locks = DefaultData::count_not_processed_locks(ctx).await;
        assert_eq!(1, locks);

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_ignore_clear_outbox_lock_when_is_schedule_is_not_on_time(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        let custom_resources = OutboxProcessorResources::new(ctx.resources.postgres_pool.clone(), ctx.resources.sqs_client.clone(), ctx.resources.sns_client.clone())
            .with_scheduled_clear_locked_partition(true);

        DefaultData::create_cleaner_schedule(ctx, "0 */2 * * * *").await;

        DefaultData::create_lock(ctx, true).await;
        DefaultData::create_lock(ctx, false).await;
        DefaultData::create_lock(ctx, true).await;

        let _ = OutboxProcessor::one_shot_processed_locked_cleaner(&custom_resources).await;

        let locks = DefaultData::count_processed_locks(ctx).await;
        assert_eq!(2, locks);

        let locks = DefaultData::count_not_processed_locks(ctx).await;
        assert_eq!(1, locks);

        Ok(())
    }
}
