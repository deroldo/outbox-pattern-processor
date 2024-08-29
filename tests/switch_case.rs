mod commons;

#[cfg(test)]
mod test {
    use crate::commons::{DefaultData, HttpGatewayMock, TestContext};
    use outbox_pattern_processor::domain::destination::http_destination::HttpDestination;
    use outbox_pattern_processor::domain::destination::outbox_destination::OutboxDestination;
    use outbox_pattern_processor::domain::destination::sns_destination::SnsDestination;
    use outbox_pattern_processor::domain::destination::sqs_destination::SqsDestination;
    use outbox_pattern_processor::outbox_processor::OutboxProcessor;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::env;
    use test_context::test_context;

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_batch_limit(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "2");

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_none());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_all(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "2");

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;
        let _ = OutboxProcessor::run(&ctx.app_state).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_all_one_shot(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_ignoring_fails(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

        let outbox_1 = DefaultData::create_default_http_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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
    async fn should_process_partition_with_one_shot(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_none());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_partition_concurrent_shots(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "30");

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        for _ in 0..10 {
            DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        }
        for _ in 0..10 {
            let other_outbox = DefaultData::create_default_http_outbox_success(ctx).await;
            HttpGatewayMock::default_mock(ctx, &other_outbox).await;
        }

        let _ = tokio::join!(OutboxProcessor::run(&ctx.app_state), OutboxProcessor::run(&ctx.app_state),);

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(21, stored_outboxes.len());

        let outboxes_processed = DefaultData::find_all_outboxes_processed(ctx).await;
        assert_eq!(11, outboxes_processed.len());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_partition_with_two_shots(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

        let outbox_1 = DefaultData::create_default_http_outbox_success(ctx).await;
        let outbox_2 = DefaultData::create_http_outbox_success_with_partition_key(ctx, outbox_1.partition_key).await;
        let outbox_3 = DefaultData::create_default_http_outbox_success(ctx).await;

        HttpGatewayMock::default_mock(ctx, &outbox_1).await;
        HttpGatewayMock::default_mock(ctx, &outbox_2).await;
        HttpGatewayMock::default_mock(ctx, &outbox_3).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;
        let _ = OutboxProcessor::run(&ctx.app_state).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(3, stored_outboxes.len());

        let stored_outbox_1 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_1.idempotent_key).unwrap();
        assert!(stored_outbox_1.processed_at.is_some());

        let stored_outbox_2 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_2.idempotent_key).unwrap();
        assert!(stored_outbox_2.processed_at.is_some());

        let stored_outbox_3 = stored_outboxes.iter().find(|it| it.idempotent_key == outbox_3.idempotent_key).unwrap();
        assert!(stored_outbox_3.processed_at.is_some());

        Ok(())
    }

    #[test_context(TestContext)]
    #[serial]
    #[tokio::test]
    async fn should_process_http_put(ctx: &mut TestContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        DefaultData::clear(ctx).await;

        env::set_var("OUTBOX_QUERY_LIMIT", "2");

        let outbox = DefaultData::create_http_outbox_success(ctx, "PUT").await;

        HttpGatewayMock::mock_put(ctx, &outbox).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "2");

        let outbox = DefaultData::create_http_outbox_success(ctx, "PATCH").await;

        HttpGatewayMock::mock_patch(ctx, &outbox).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "2");

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

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

        let outbox_1 = DefaultData::create_default_sns_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_sns_outbox_success(ctx).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

        let outbox_1 = DefaultData::create_default_sqs_outbox_failed(ctx).await;
        let outbox_2 = DefaultData::create_default_sqs_outbox_success(ctx).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

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
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

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
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

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
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

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

        env::set_var("OUTBOX_QUERY_LIMIT", "5");

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
        )
        .await;

        HttpGatewayMock::default_mock(ctx, &outbox).await;

        let _ = OutboxProcessor::run(&ctx.app_state).await;

        let stored_outboxes = DefaultData::find_all_outboxes(ctx).await;
        assert_eq!(1, stored_outboxes.len());

        let stored_outbox = stored_outboxes[0].clone();
        assert!(stored_outbox.processed_at.is_none());

        Ok(())
    }
}
