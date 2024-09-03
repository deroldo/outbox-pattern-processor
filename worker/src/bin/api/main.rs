use aws_config::BehaviorVersion;
use outbox_pattern_processor::aws::{SnsClient, SqsClient};
use outbox_pattern_processor::environment::Environment;
use outbox_pattern_processor::outbox_processor::OutboxProcessor;
use outbox_pattern_processor::outbox_resources::OutboxProcessorResources;
use outbox_pattern_processor::shutdown::Shutdown;
use outbox_pattern_processor_worker::infra::database::Database;
use outbox_pattern_processor_worker::routes::Routes;
use sqlx::{Pool, Postgres};
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::log::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use wg::WaitGroup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    let rust_log = Environment::string("RUST_LOG", "INFO,sqlx::postgres::notice=WARN,sqlx::query=WARN");
    env::set_var("RUST_LOG", rust_log);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(Box::new(tracing_subscriber::fmt::layer().with_writer(non_blocking)))
        .init();

    info!("Starting...");

    let wait_group = WaitGroup::new();

    let db_config = Database::from_env();
    let postgres_pool = db_config.create_db_pool().await?;

    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let sqs_client = SqsClient::new(&aws_config).await;
    let sns_client = SnsClient::new(&aws_config).await;

    tokio::spawn(init_http_server(wait_group.add(1)));
    tokio::spawn(init_outbox(postgres_pool, sqs_client, sns_client, wait_group.add(1)));

    wait_group.wait();

    info!("Stopped!");

    Ok(())
}

async fn init_http_server(wait_group: WaitGroup) {
    info!("Starting http server...");
    let routes = Routes::routes().await;

    let addr = SocketAddr::from(([0, 0, 0, 0], 9095));

    if let Ok(listener) = TcpListener::bind(addr).await {
        info!("Running http server...");
        let _ = axum::serve(listener, routes).with_graceful_shutdown(Shutdown::signal("Stopping http server...")).await;
    }

    wait_group.done();

    info!("Http server stopped!");
}

async fn init_outbox(
    postgres_pool: Pool<Postgres>,
    sqs_client: SqsClient,
    sns_client: SnsClient,
    wait_group: WaitGroup,
) {
    let outbox_processor_resources = OutboxProcessorResources::new(postgres_pool, Some(sqs_client), Some(sns_client));

    let _ = OutboxProcessor::new(outbox_processor_resources)
        .with_graceful_shutdown(Shutdown::signal("Stopping outbox processor..."))
        .init()
        .await;

    wait_group.done();
}
