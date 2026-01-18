use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{error::ApiError, AppState};

/// Rate limit middleware (simplified implementation)
/// In production, use a proper rate limiting library like governor
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract IP address (simplified - should handle X-Forwarded-For)
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let key = format!("rate_limit:{}", ip);
    let limit: usize = 100; // requests per minute
    let window: u64 = 60; // seconds

    let mut conn = state.redis.clone();

    // Get current count
    let count: Option<usize> = conn.get(&key).await.ok();

    match count {
        Some(c) if c >= limit => Err(StatusCode::TOO_MANY_REQUESTS),
        Some(c) => {
            // Increment counter
            let _: () = conn.incr(&key, 1).await.ok().unwrap_or(());
            Ok(next.run(req).await)
        }
        None => {
            // Set initial counter with expiry
            let _: () = conn.set_ex(&key, 1, window).await.ok().unwrap_or(());
            Ok(next.run(req).await)
        }
    }
}
