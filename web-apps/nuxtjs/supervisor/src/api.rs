use axum::{
    Json, Router, http::StatusCode, response::{IntoResponse, Response}, routing::post, extract::Query
};
use serde::{Deserialize, Serialize};

pub async fn start_api() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/webhook/update", post(webhook_update));

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

    let response = WebhookUpdateResponse {
        success: true,
        message: format!("Update received successfully"),
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Serialize)]
struct WebhookUpdateResponse {
    success: bool,
    message: String
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}