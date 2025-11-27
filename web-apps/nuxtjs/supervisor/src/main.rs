// import start_api from ./api.ra
mod api;
mod proxy;

// fn init_tracing() {

//     // Ignore errors when the global subscriber is already set (e.g. in tests).
//     let _ = tracing_subscriber::fmt::try_init();
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // init_tracing();

    let proxy_task = tokio::task::spawn_blocking(|| proxy::start_proxy());
    let api_task = tokio::spawn(async {
        api::start_api().await;
    });

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
