use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
    pub media_service_url: String,
    pub ai_service_url: String,
    pub alerts_service_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiry_minutes: env::var("JWT_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()?,
            refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            media_service_url: env::var("MEDIA_SERVICE_URL")
                .unwrap_or_else(|_| "http://media-service:8081".to_string()),
            ai_service_url: env::var("AI_SERVICE_URL")
                .unwrap_or_else(|_| "http://ai-service:8082".to_string()),
            alerts_service_url: env::var("ALERTS_SERVICE_URL")
                .unwrap_or_else(|_| "http://alerts-service:8083".to_string()),
        })
    }
}
