use axum::{
    Json, Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use crate::instance_handler;

pub async fn start_api() {
    let app = Router::new().route("/_supervisor/webhook/update", post(webhook_update));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:19180")
        .await
        .unwrap();

    tracing::info!("api listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct AuthQuery {
    apikey: Option<String>,
}

async fn webhook_update(Query(query): Query<AuthQuery>) -> Response {
    let expected_key = std::env::var("SUPERVISOR_API_KEY").ok();

    // Auth check via `apikey` query parameter
    let authorized: bool = match (expected_key.as_deref(), query.apikey.as_deref()) {
        (Some(expected), Some(provided)) if !expected.is_empty() && provided == expected => true,
        _ => false,
    };

    if !authorized {
        let body = ErrorResponse {
            success: false,
            message: "Unauthorized: invalid or missing API key".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
    }

    // shedule update
    instance_handler::InstanceHandler::on_update().await;

    let response = WebhookUpdateResponse {
        success: true,
        message: "Update was added to the queue and will be processed shortly.".to_string(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Serialize)]
struct WebhookUpdateResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}
