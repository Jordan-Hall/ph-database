use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::{
    error::ApiError,
    models::Claims,
    AppState,
};

pub struct AuthLayer;

/// Extract and verify JWT token from Authorization header
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

    // Verify token
    let claims = jsonwebtoken::decode::<Claims>(
        &token,
        &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| ApiError::Authentication(format!("Invalid token: {}", e)))?
    .claims;

    // Add claims to request extensions
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Check if user has required role
pub fn require_role(required_role: &'static str) -> impl Fn(Request, Next) -> futures::future::BoxFuture<'static, Result<Response, StatusCode>> + Clone {
    move |req: Request, next: Next| {
        Box::pin(async move {
            let claims = req.extensions().get::<Claims>().cloned();

            match claims {
                Some(claims) if claims.roles.contains(&required_role.to_string()) => {
                    Ok(next.run(req).await)
                }
                Some(_) => Err(StatusCode::FORBIDDEN),
                None => Err(StatusCode::UNAUTHORIZED),
            }
        })
    }
}
