use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn api_key_auth(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let api_key = std::env::var("KANBAN_API_KEY").ok();

    if api_key.is_none() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "KANBAN_API_KEY environment variable not set".to_string(),
            }),
        ));
    }

    let expected_key = api_key.unwrap();
    let provided_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if provided_key.is_none() || provided_key.as_ref().unwrap() != &expected_key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or missing API key".to_string(),
            }),
        ));
    }

    Ok(next.run(req).await)
}
