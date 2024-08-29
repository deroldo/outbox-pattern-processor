use outbox_pattern_processor::infra::environment::Environment;
use outbox_pattern_processor::outbox_processor::OutboxProcessor;
use outbox_pattern_processor::routes::Routes;
use outbox_pattern_processor::state::AppState;
use std::cmp::Ordering;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::log::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use wg::WaitGroup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(Box::new(tracing_subscriber::fmt::layer().with_writer(non_blocking)))
        .init();

    info!("Starting...");

    let wait_group = WaitGroup::new();

    let app_state = AppState::new().await?;

    tokio::spawn(init_http_server(app_state.clone(), wait_group.add(1)));
    tokio::spawn(init_outbox_processor(app_state.clone(), wait_group.add(1)));

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

async fn init_outbox_processor(
    app_state: AppState,
    wait_group: WaitGroup,
) {
    info!("Starting outbox processor...");
    let mut test_processor_executions = Environment::i32("TEST_PROCESSOR_EXECUTIONS", -1);

    info!("Running outbox processor...");
    loop {
        tokio::select! {
            Ok(_) = OutboxProcessor::run(&app_state) => {
                match test_processor_executions.cmp(&0) {
                    Ordering::Greater => {
                        test_processor_executions -= 1;
                    },
                    Ordering::Equal => {
                        break;
                    }
                    Ordering::Less => {},
                }
            }
            Err(error) = OutboxProcessor::run(&app_state) => {
                error!("Outbox processor failed with error: {}", error.to_string());
                match test_processor_executions.cmp(&0) {
                    Ordering::Greater => {
                        test_processor_executions -= 1;
                    },
                    Ordering::Equal => {
                        break;
                    }
                    Ordering::Less => {},
                }

            }
            _ = shutdown_signal("Stopping outbox processor...") => {
                break;
            }
        }
    }

    wait_group.done();

    info!("Outbox processor stopped!");
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
