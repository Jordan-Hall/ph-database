use crate::{
    error::{ApiError, ApiResult},
    models::Claims,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub fn generate_token(
    user_id: &str,
    email: &str,
    roles: &[String],
    secret: &str,
    expiry_minutes: i64,
) -> ApiResult<String> {
    let now = Utc::now();
    let exp = (now + Duration::minutes(expiry_minutes))
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        roles: roles.to_vec(),
        exp,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to generate token: {}", e)))
}

pub fn verify_token(token: &str, secret: &str) -> ApiResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| ApiError::Authentication(format!("Invalid token: {}", e)))
}
