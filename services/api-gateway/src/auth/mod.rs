pub mod jwt;
pub mod password;

pub use jwt::*;
pub use password::*;

use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserInfo},
};
use serde_json::json;

pub struct AuthService {
    db: Database,
    db_url: String,
}

impl AuthService {
    pub fn new(db: Database, db_url: String) -> Self {
        Self { db, db_url }
    }

    /// Register a new user using SurrealDB's native SIGNUP
    pub async fn register(&self, req: RegisterRequest) -> ApiResult<AuthResponse> {
        // Use SurrealDB's native SIGNUP through the user_scope
        let token = self
            .db
            .signup(req.username.clone(), req.email.clone(), req.password)
            .await
            .map_err(|e| {
                tracing::error!("Signup failed: {}", e);
                ApiError::Conflict("Email already registered or invalid data".to_string())
            })?;

        // Fetch the created user info
        let users: Vec<User> = self
            .db
            .query(&format!("SELECT * FROM user WHERE email = '{}'", req.email))
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch user after signup: {}", e);
                ApiError::Internal(anyhow::anyhow!("Failed to fetch user data"))
            })?;

        let user = users
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("User created but not found")))?;

        Ok(AuthResponse {
            access_token: token.clone(),
            refresh_token: token, // SurrealDB manages token refresh internally
            user: UserInfo {
                id: user.id.unwrap_or_default(),
                username: user.username,
                email: user.email,
                roles: user.roles,
            },
        })
    }

    /// Login user using SurrealDB's native SIGNIN
    pub async fn login(&self, req: LoginRequest) -> ApiResult<AuthResponse> {
        // Use SurrealDB's native SIGNIN through the user_scope
        let token = self
            .db
            .signin(req.email.clone(), req.password)
            .await
            .map_err(|e| {
                tracing::error!("Signin failed: {}", e);
                ApiError::Authentication("Invalid credentials".to_string())
            })?;

        // Fetch user info
        let users: Vec<User> = self
            .db
            .query(&format!("SELECT * FROM user WHERE email = '{}'", req.email))
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch user after signin: {}", e);
                ApiError::Internal(anyhow::anyhow!("Failed to fetch user data"))
            })?;

        let user = users
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::Authentication("User not found".to_string()))?;

        // Update last login timestamp
        let _: Vec<serde_json::Value> = self
            .db
            .query(&format!(
                "UPDATE {} SET last_login = time::now()",
                user.id.as_ref().unwrap_or(&"".to_string())
            ))
            .await
            .unwrap_or_default();

        Ok(AuthResponse {
            access_token: token.clone(),
            refresh_token: token, // SurrealDB manages token refresh internally
            user: UserInfo {
                id: user.id.unwrap_or_default(),
                username: user.username,
                email: user.email,
                roles: user.roles,
            },
        })
    }

    /// Verify SurrealDB token (used by middleware)
    pub async fn verify_token(&self, token: &str) -> ApiResult<User> {
        // Verify token using SurrealDB's native authentication
        let user_data = self
            .db
            .verify_token(token, &self.db_url)
            .await
            .map_err(|e| {
                tracing::error!("Token verification failed: {}", e);
                ApiError::Authentication("Invalid or expired token".to_string())
            })?;

        // Parse user data from the authentication context
        let user: User = serde_json::from_value(user_data).map_err(|e| {
            tracing::error!("Failed to parse user from token data: {}", e);
            ApiError::Authentication("Invalid token data".to_string())
        })?;

        Ok(user)
    }
}
