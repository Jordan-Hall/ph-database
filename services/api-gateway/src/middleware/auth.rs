use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::{
    auth::AuthService,
    error::ApiError,
    models::User,
    AppState,
};

pub struct AuthLayer;

/// Extract and verify SurrealDB token from Authorization header
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::Authentication("Missing authorization token".to_string()))?;

    // Verify SurrealDB token and get user
    let auth_service = AuthService::new(state.db.clone());
    let user = auth_service.verify_token(&token).await?;

    // Add user to request extensions (replaces Claims with full User)
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

/// Check if user has required role (middleware function)
/// TODO: Implement as a proper layer/middleware
pub async fn require_role(
    user: &User,
    required_role: &str,
) -> Result<(), StatusCode> {
    if user.roles.contains(&required_role.to_string()) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
