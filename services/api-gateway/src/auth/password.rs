use crate::error::{ApiError, ApiResult};
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> ApiResult<String> {
    hash(password, DEFAULT_COST)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> ApiResult<bool> {
    verify(password, hash)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Password verification failed: {}", e)))
}
