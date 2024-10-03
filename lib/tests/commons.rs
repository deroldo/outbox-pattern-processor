use aws_config::BehaviorVersion;
use aws_sdk_sns::operation::create_topic::CreateTopicOutput;
use aws_sdk_sqs::operation::create_queue::CreateQueueOutput;
use outbox_pattern_processor::aws::{SnsClient, SqsClient};
use outbox_pattern_processor::http_destination::HttpDestination;
use outbox_pattern_processor::outbox::Outbox;
use outbox_pattern_processor::outbox_destination::OutboxDestination;
use outbox_pattern_processor::outbox_resources::OutboxProcessorResources;
use outbox_pattern_processor::sns_destination::SnsDestination;
use outbox_pattern_processor::sqs_destination::SqsDestination;
use rand::Rng;
use serde_json::{json, Value};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::env;
use std::net::{SocketAddr, TcpListener};
use test_context::AsyncTestContext;
use uuid::Uuid;
use wiremock::matchers::{body_json_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[allow(dead_code)]
pub struct TestContext {
    pub resources: OutboxProcessorResources,
    mock_server: MockServer,
    pub gateway_uri: String,
    pub queue_url: String,
    pub topic_arn: String,
}

impl AsyncTestContext for TestContext {
    async fn setup() -> Self {
        env::set_var("AWS_ACCESS_KEY_ID", "outbox-pattern-processor");
        env::set_var("AWS_SECRET_ACCESS_KEY", "outbox-pattern-processor");
        env::set_var("LOCAL_ENDPOINT", "http://localhost:4566");
        env::set_var("LOCAL_REGION", "us-east-1");

        let mock_server = Infrastructure::init_mock_server().await;

        let postgres_pool = Infrastructure::init_database().await;

        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let sqs_client = SqsClient::new(&aws_config).await;
        let sns_client = SnsClient::new(&aws_config).await;

        let resources = OutboxProcessorResources::new(postgres_pool, Some(sqs_client), Some(sns_client));

        let gateway_uri = mock_server.uri();
        let queue_url = Infrastructure::init_sqs(&resources).await.queue_url.unwrap();
        let topic_arn = Infrastructure::init_sns(&resources).await.topic_arn.unwrap();

        Self {
            resources,
            mock_server,
            gateway_uri,
            queue_url,
            topic_arn,
        }
    }
}

pub struct Infrastructure;

impl Infrastructure {
    async fn init_database() -> Pool<Postgres> {
        PgPoolOptions::new()
            .min_connections(1)
            .max_connections(10)
            .test_before_acquire(true)
            .connect_with(
                PgConnectOptions::new()
                    .host("localhost")
                    .database("local")
                    .username("local")
                    .password("local")
                    .port(5432)
                    .application_name("outbox-pattern-processor"),
            )
            .await
            .unwrap()
    }

    async fn init_sqs(resources: &OutboxProcessorResources) -> CreateQueueOutput {
        resources.sqs_client.clone().unwrap().client.create_queue().queue_name("queue").send().await.unwrap()
    }

    async fn init_sns(resources: &OutboxProcessorResources) -> CreateTopicOutput {
        resources.sns_client.clone().unwrap().client.create_topic().name("topic").send().await.unwrap()
    }

    async fn init_mock_server() -> MockServer {
        for _ in 1..10 {
            let port = rand::thread_rng().gen_range(51000..54000);
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            if let Ok(listener) = TcpListener::bind(addr) {
                return MockServer::builder().listener(listener).start().await;
            }
        }

        panic!("Failed to create mock server");
    }
}

pub struct DefaultData;

impl DefaultData {
    pub async fn create_default_http_outbox_success(ctx: &mut TestContext) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::HttpDestination(HttpDestination {
                url: format!("{}/success", ctx.gateway_uri),
                headers: None,
                method: None,
            })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_default_scheduled(
        ctx: &mut TestContext,
        process_after: DateTime<Utc>,
    ) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::HttpDestination(HttpDestination {
                url: format!("{}/success", ctx.gateway_uri),
                headers: None,
                method: None,
            })],
            None,
            None,
            Some(process_after),
        )
        .await
    }

    pub async fn create_default_http_outbox_failed(ctx: &mut TestContext) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::HttpDestination(HttpDestination {
                url: format!("{}/failed", ctx.gateway_uri),
                headers: None,
                method: None,
            })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_http_outbox_success_with_partition_key(
        ctx: &mut TestContext,
        partition_key: Uuid,
    ) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            Some(partition_key),
            vec![OutboxDestination::HttpDestination(HttpDestination {
                url: format!("{}/success", ctx.gateway_uri),
                headers: None,
                method: None,
            })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_http_outbox_success(
        ctx: &mut TestContext,
        method: &str,
    ) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::HttpDestination(HttpDestination {
                url: format!("{}/success", ctx.gateway_uri),
                headers: None,
                method: Some(method.to_string()),
            })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_http_outbox_with_headers(
        ctx: &mut TestContext,
        http_headers_map: HashMap<String, String>,
        outbox_headers_map: HashMap<String, String>,
    ) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::HttpDestination(HttpDestination {
                url: format!("{}/success", ctx.gateway_uri),
                headers: Some(http_headers_map),
                method: None,
            })],
            Some(outbox_headers_map),
            None,
            None,
        )
        .await
    }

    pub async fn create_default_sqs_outbox_success(ctx: &mut TestContext) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::SqsDestination(SqsDestination { queue_url: ctx.queue_url.clone() })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_default_sqs_outbox_failed(ctx: &mut TestContext) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::SqsDestination(SqsDestination {
                queue_url: "https://invalid.queue.com".to_string(),
            })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_default_sns_outbox_success(ctx: &mut TestContext) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::SnsDestination(SnsDestination { topic_arn: ctx.topic_arn.clone() })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_default_sns_outbox_failed(ctx: &mut TestContext) -> Outbox {
        Self::create_outbox(
            ctx,
            None,
            None,
            vec![OutboxDestination::SnsDestination(SnsDestination {
                topic_arn: "invalid::arn".to_string(),
            })],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_outbox(
        ctx: &mut TestContext,
        idempotent_key: Option<Uuid>,
        partition_key: Option<Uuid>,
        destinations: Vec<OutboxDestination>,
        headers: Option<HashMap<String, String>>,
        payload: Option<String>,
        process_after: Option<DateTime<Utc>>,
    ) -> Outbox {
        if let Some(date_to_process) = process_after {
            let sql = r#"
            insert into outbox
                (idempotent_key, partition_key, destinations, headers, payload, process_after)
            values
                ($1, $2, $3, $4, $5, $6)
            returning *
            "#;

            sqlx::query_as(sql)
                .bind(idempotent_key.unwrap_or(Uuid::now_v7()))
                .bind(partition_key.unwrap_or(Uuid::now_v7()))
                .bind(Json(destinations))
                .bind(headers.map(|it| Some(Json(it))))
                .bind(payload.unwrap_or(json!({"foo":"bar"}).to_string()))
                .bind(date_to_process)
                .fetch_one(&ctx.resources.postgres_pool)
                .await
                .unwrap()
        } else {
            let sql = r#"
            insert into outbox
                (idempotent_key, partition_key, destinations, headers, payload)
            values
                ($1, $2, $3, $4, $5)
            returning *
            "#;

            sqlx::query_as(sql)
                .bind(idempotent_key.unwrap_or(Uuid::now_v7()))
                .bind(partition_key.unwrap_or(Uuid::now_v7()))
                .bind(Json(destinations))
                .bind(headers.map(|it| Some(Json(it))))
                .bind(payload.unwrap_or(json!({"foo":"bar"}).to_string()))
                .fetch_one(&ctx.resources.postgres_pool)
                .await
                .unwrap()
        }
    }

    pub async fn find_all_outboxes(ctx: &mut TestContext) -> Vec<Outbox> {
        let sql = r#"
        select * 
        from outbox
        "#;

        sqlx::query_as(sql).fetch_all(&ctx.resources.postgres_pool).await.unwrap()
    }

    pub async fn find_all_outboxes_processed(ctx: &mut TestContext) -> Vec<Outbox> {
        let sql = r#"
        select *
        from outbox
        where processed_at is not null
        "#;

        sqlx::query_as(sql).fetch_all(&ctx.resources.postgres_pool).await.unwrap()
    }

    pub async fn clear(ctx: &mut TestContext) {
        let _ = sqlx::query("delete from outbox").execute(&ctx.resources.postgres_pool).await;
        let _ = sqlx::query("delete from outbox_lock").execute(&ctx.resources.postgres_pool).await;
    }
}

pub struct HttpGatewayMock;

impl HttpGatewayMock {
    pub async fn default_mock(
        ctx: &mut TestContext,
        outbox: &Outbox,
    ) {
        Self::mock(ctx, outbox, "POST", None, None).await;
    }

    pub async fn mock_put(
        ctx: &mut TestContext,
        outbox: &Outbox,
    ) {
        Self::mock(ctx, outbox, "PUT", None, None).await;
    }

    pub async fn mock_patch(
        ctx: &mut TestContext,
        outbox: &Outbox,
    ) {
        Self::mock(ctx, outbox, "PATCH", None, None).await;
    }

    pub async fn mock_with_headers(
        ctx: &mut TestContext,
        outbox: &Outbox,
        headers_map: HashMap<String, String>,
    ) {
        Self::mock(ctx, outbox, "POST", None, Some(headers_map)).await;
    }

    async fn mock(
        ctx: &mut TestContext,
        outbox: &Outbox,
        method_name: &str,
        payload: Option<Value>,
        headers_map: Option<HashMap<String, String>>,
    ) {
        let mut mock_builder = Mock::given(method(method_name)).and(body_json_string(payload.unwrap_or(json!({"foo":"bar"})).to_string()));

        match headers_map {
            None => {},
            Some(headers) => {
                for (key, value) in headers {
                    mock_builder = mock_builder.and(header(key.as_str(), value.as_str()));
                }
            },
        }

        mock_builder
            .and(header("x-idempotent-key", outbox.idempotent_key.to_string()))
            .and(path("/success"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&ctx.mock_server)
            .await;

        Mock::given(method(method_name))
            .and(header("x-idempotent-key", outbox.idempotent_key.to_string()))
            .and(path("/failed"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&ctx.mock_server)
            .await;
    }
}
