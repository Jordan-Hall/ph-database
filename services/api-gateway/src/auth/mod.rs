pub mod jwt;
pub mod password;

pub use jwt::*;
pub use password::*;

use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{AuthResponse, Claims, LoginRequest, RegisterRequest, User, UserInfo, UserStatus},
};
use chrono::Utc;
use uuid::Uuid;

pub struct AuthService {
    db: Database,
    jwt_secret: String,
    jwt_expiry_minutes: i64,
    refresh_token_expiry_days: i64,
}

impl AuthService {
    pub fn new(
        db: Database,
        jwt_secret: String,
        jwt_expiry_minutes: i64,
        refresh_token_expiry_days: i64,
    ) -> Self {
        Self {
            db,
            jwt_secret,
            jwt_expiry_minutes,
            refresh_token_expiry_days,
        }
    }

    pub async fn register(&self, req: RegisterRequest) -> ApiResult<AuthResponse> {
        // Check if user already exists
        let existing: Vec<User> = self
            .db
            .query(&format!(
                "SELECT * FROM user WHERE email = '{}'",
                req.email
            ))
            .await?;

        if !existing.is_empty() {
            return Err(ApiError::Conflict("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = hash_password(&req.password)?;

        // Create user
        let user = User {
            id: None,
            username: req.username.clone(),
            email: req.email.clone(),
            password_hash,
            roles: vec!["user".to_string()],
            status: UserStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            mfa_enabled: false,
        };

        let created: Vec<User> = self.db.create("user", user).await?;
        let user = created
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("Failed to create user")))?;

        // Generate tokens
        let user_id = user.id.clone().unwrap();
        let access_token = generate_token(
            &user_id,
            &user.email,
            &user.roles,
            &self.jwt_secret,
            self.jwt_expiry_minutes,
        )?;
        let refresh_token = Uuid::new_v4().to_string();

        // Store refresh token (simplified - should be in separate table)
        // TODO: Implement proper refresh token storage

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: UserInfo {
                id: user_id,
                username: user.username,
                email: user.email,
                roles: user.roles,
            },
        })
    }

    pub async fn login(&self, req: LoginRequest) -> ApiResult<AuthResponse> {
        // Find user by email
        let users: Vec<User> = self
            .db
            .query(&format!("SELECT * FROM user WHERE email = '{}'", req.email))
            .await?;

        let user = users
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::Authentication("Invalid credentials".to_string()))?;

        // Verify password
        if !verify_password(&req.password, &user.password_hash)? {
            return Err(ApiError::Authentication("Invalid credentials".to_string()));
        }

        // Check if user is active
        if user.status != UserStatus::Active {
            return Err(ApiError::Authentication(
                "Account is not active".to_string(),
            ));
        }

        // Generate tokens
        let user_id = user.id.clone().unwrap();
        let access_token = generate_token(
            &user_id,
            &user.email,
            &user.roles,
            &self.jwt_secret,
            self.jwt_expiry_minutes,
        )?;
        let refresh_token = Uuid::new_v4().to_string();

        // Update last login
        // TODO: Implement last login update

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: UserInfo {
                id: user_id,
                username: user.username,
                email: user.email,
                roles: user.roles,
            },
        })
    }

    pub fn verify_token(&self, token: &str) -> ApiResult<Claims> {
        verify_token(token, &self.jwt_secret)
    }
}
