// import start_api from ./api.ra
pub mod api;
pub mod instance_handler;
pub mod proxy;
pub mod utils;
pub mod runtime_cli;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // fallback to debug level

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE) // optional, log spans
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();

    instance_handler::InstanceHandler::startup().await;

    tokio::spawn(runtime_cli::start());

    let proxy_task = tokio::task::spawn_blocking(|| proxy::start_proxy());
    let api_task = tokio::spawn(async {
        api::start_api().await;
    });

    tracing::info!(target: "supervisor", "supervisor started successfully");

    tokio::select! {
        proxy_result = proxy_task => {
            match proxy_result {
                Ok(Ok(())) => tracing::info!(target: "supervisor", "proxy task exited cleanly"),
                Ok(Err(err)) => tracing::error!(target: "supervisor", error = %err, "proxy task exited with error"),
                Err(err) => tracing::error!(target: "supervisor", error = %err, "proxy task panicked"),
            }
        }
        api_result = api_task => {
            match api_result {
                Ok(()) => tracing::info!(target: "supervisor", "api task exited"),
                Err(err) => tracing::error!(target: "supervisor", error = %err, "api task panicked"),
            }
        }
    }
}
