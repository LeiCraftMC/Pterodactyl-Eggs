use axum::{
    extract::{Path, State},
    http::{
        uri::Uri,
        Request, StatusCode,
    },
    routing::{get, post},
    Json, Router,
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tracing::{info, error};

// --- Application State and Configuration ---

// The shared state for our application
struct AppState {
    // List of backend service URLs for load balancing
    backends: Vec<Url>,
    // HTTP client to forward requests
    client: Client,
    // Simple counter to implement a round-robin load balancing strategy
    next_backend_index: std::sync::atomic::AtomicUsize,
}

// Wrap AppState in an Arc for shared, thread-safe access
type SharedState = Arc<AppState>;

// --- Webhook Structs ---

#[derive(Debug, Deserialize, Serialize)]
struct WebhookPayload {
    // A simple example field for the payload
    message: String,
    timestamp: u64,
}

#[derive(Debug, Serialize)]
struct WebhookResponse {
    status: String,
    received_data: WebhookPayload,
}

// --- Handler Functions ---

// The webhook handler for POST requests to /_supervisor/webhook/update
async fn supervisor_webhook_update(
    // The framework handles deserializing the JSON body into our struct
    Json(payload): Json<WebhookPayload>,
) -> (StatusCode, Json<WebhookResponse>) {
    info!("Received webhook update: {:?}", payload);

    // TODO: Add your custom logic here (e.g., triggering a configuration reload,
    // logging, sending an event, etc.)

    let response = WebhookResponse {
        status: "Success".to_string(),
        received_data: payload,
    };

    // Return a 200 OK status with the JSON response
    (StatusCode::OK, Json(response))
}

// The main reverse proxy/load balancing handler for all other paths
async fn reverse_proxy_handler(
    // Get the shared application state
    State(state): State<SharedState>,
    // Take the full original request
    mut req: Request<axum::body::Body>,
) -> Result<axum::response::Response, StatusCode> {
    // 1. **Load Balancing: Round Robin Strategy**
    let index = state.next_backend_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let backend_url = state.backends.get(index % state.backends.len())
        .ok_or_else(|| {
            error!("Load balancer configuration error: No backends available.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 2. **Construct New URI for Backend**
    let mut uri_parts = req.uri().clone().into_parts();
    
    // Safety check: ensure path and query exist before setting them
    let path_and_query = uri_parts.path_and_query.take()
        .map(|pq| pq.to_string())
        .unwrap_or_else(|| "/".to_string());

    // Construct the final backend URL (e.g., http://backend1:8080/original/path?query=params)
    let new_uri_string = format!("{}{}", backend_url, path_and_query);
    
    // Set the request's URI to the target backend's URI
    *req.uri_mut() = new_uri_string.parse::<Uri>().map_err(|e| {
        error!("Failed to parse new URI: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. **Forward Request and Handle Response**
    info!("Proxying request to: {}", req.uri());

    match state.client.execute(req.try_into().unwrap()).await {
        Ok(backend_response) => {
            // Success: Map the reqwest response to an axum response
            Ok(backend_response.into())
        }
        Err(e) => {
            // Failure: e.g., backend service is down, network error
            error!("Error communicating with backend service: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

// --- Main Function ---

#[tokio::main]
async fn main() {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    // --- Configuration ---
    let backend_urls = vec![
        Url::parse("http://127.0.0.1:8081").expect("Invalid backend URL 1"),
        Url::parse("http://127.0.0.1:8082").expect("Invalid backend URL 2"),
        // Add more backends here
    ];
    let proxy_listen_addr: SocketAddr = "127.0.0.1:3000".parse().expect("Invalid listen address");

    // --- App State Initialization ---
    let shared_state = SharedState::new(AppState {
        backends: backend_urls,
        client: Client::new(), // reqwest client for proxying
        next_backend_index: std::sync::atomic::AtomicUsize::new(0),
    });

    // --- Router Definition ---
    let app = Router::new()
        // 1. Webhook Endpoint
        .route("/_supervisor/webhook/update", post(supervisor_webhook_update))
        // 2. Reverse Proxy/Load Balancing: Catch-all route
        // This MUST be the last route. It catches all GET, POST, PUT, etc.,
        // that haven't been matched by the specific webhook route above.
        .fallback(reverse_proxy_handler)
        // Add the shared state to the router
        .with_state(shared_state);

    // --- Server Startup ---
    info!("🚀 Reverse Proxy listening on {}", proxy_listen_addr);
    info!("Backend services: {:?}", &shared_state.backends);
    info!("Webhook endpoint: POST http://{}/_supervisor/webhook/update", proxy_listen_addr);

    // Start the server
    axum::Server::bind(&proxy_listen_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}