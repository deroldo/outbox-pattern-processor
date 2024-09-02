use outbox_pattern_processor::infra::environment::Environment;
use outbox_pattern_processor::outbox_processor::{OutboxProcessor, OutboxProcessorResources};
use outbox_pattern_processor_worker::routes::Routes;
use outbox_pattern_processor_worker::state::AppState;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
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

    let app_state = AppState::new().await?;

    tokio::spawn(init_http_server(app_state.clone(), wait_group.add(1)));
    tokio::spawn(init_outbox(app_state.clone(), wait_group.add(1)));

    wait_group.wait();

    info!("Stopped!");

    Ok(())
}

async fn init_http_server(
    app_state: AppState,
    wait_group: WaitGroup,
) {
    info!("Starting http server...");
    let routes = Routes::routes(&app_state).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], 9095));

    if let Ok(listener) = TcpListener::bind(addr).await {
        info!("Running http server...");
        let _ = axum::serve(listener, routes).with_graceful_shutdown(shutdown_signal("Stopping http server...")).await;
    }

    wait_group.done();

    info!("Http server stopped!");
}

async fn init_outbox(
    app_state: AppState,
    wait_group: WaitGroup,
) {
    let outbox_processor_resources = OutboxProcessorResources {
        postgres_pool: app_state.postgres_pool.clone(),
        sqs_client: app_state.sqs_client.clone(),
        sns_client: app_state.sns_client.clone(),
        http_gateway: app_state.http_gateway.clone(),
    };

    OutboxProcessor::init(outbox_processor_resources, shutdown_signal("Stopping outbox processor...")).await;

    wait_group.done();
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
