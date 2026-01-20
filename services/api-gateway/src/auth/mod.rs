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
            mfa_required: None,
        })
    }

    /// Login user using SurrealDB's native SIGNIN
    pub async fn login(&self, req: LoginRequest) -> ApiResult<AuthResponse> {
        // Use SurrealDB's native SIGNIN through the user_scope
        let token = self
            .db
            .signin(req.email.clone(), req.password.clone())
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

        // Check if MFA is enabled for this user
        if user.mfa_enabled {
            // If MFA code is not provided, return MFA required response
            if req.mfa_code.is_none() {
                return Ok(AuthResponse {
                    access_token: "".to_string(),  // Empty token
                    refresh_token: "".to_string(), // Empty token
                    user: UserInfo {
                        id: user.id.unwrap_or_default(),
                        username: user.username.clone(),
                        email: user.email.clone(),
                        roles: vec![],  // Don't expose roles until MFA verified
                    },
                    mfa_required: Some(true),
                });
            }

            // Verify MFA code
            use totp_rs::{Algorithm, Secret, TOTP};
            use bcrypt::verify;

            let user_id = user.id.clone().unwrap_or_default();

            // Fetch MFA secret
            let mfa_query = format!(
                "SELECT * FROM mfa_secret WHERE user_id = {}",
                user_id
            );

            let mfa_secrets: Vec<crate::models::MfaSecret> = self
                .db
                .query(&mfa_query)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch MFA secret: {}", e);
                    ApiError::Internal(anyhow::anyhow!("Failed to fetch MFA secret"))
                })?;

            let mfa_secret = mfa_secrets
                .into_iter()
                .next()
                .ok_or_else(|| ApiError::Authentication("MFA not properly configured".to_string()))?;

            let mfa_code = req.mfa_code.unwrap();

            // Try TOTP verification first
            let secret = Secret::Encoded(mfa_secret.secret.clone())
                .to_bytes()
                .map_err(|e| {
                    tracing::error!("Failed to decode MFA secret: {}", e);
                    ApiError::Internal(anyhow::anyhow!("Failed to decode MFA secret"))
                })?;

            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret,
                Some("PredatorHuntersDB".to_string()),
                user.email.clone(),
            ).map_err(|e| {
                tracing::error!("Failed to create TOTP: {}", e);
                ApiError::Internal(anyhow::anyhow!("Failed to create TOTP"))
            })?;

            let is_valid_totp = totp.check_current(&mfa_code).unwrap_or(false);

            // If TOTP fails, try backup codes
            let mut is_valid = is_valid_totp;
            if !is_valid {
                for hashed_code in &mfa_secret.backup_codes {
                    if verify(&mfa_code, hashed_code).unwrap_or(false) {
                        is_valid = true;
                        tracing::info!("User {} logged in with backup code", user.username);
                        // TODO: Mark backup code as used
                        break;
                    }
                }
            }

            if !is_valid {
                return Err(ApiError::Authentication("Invalid MFA code".to_string()));
            }

            tracing::info!("User {} passed MFA verification", user.username);
        }

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
            mfa_required: None,
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
