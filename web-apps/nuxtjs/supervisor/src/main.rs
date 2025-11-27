// import start_api from ./api.ra
mod api;

#[tokio::main]
async fn main() {
    // tokio::spawn(backend_server());
    api::start_api().await;
}
