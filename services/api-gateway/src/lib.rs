// API Gateway Library - Test Support Module
// This module exposes internal components for integration testing

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;

use axum::Router;
use crate::{
    config::Config,
    db::Database,
};

/// AppState for sharing across routes
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: redis::aio::ConnectionManager,
    pub config: Config,
}

/// Build application router for testing
pub async fn build_app(config: Config, db: Database, redis: redis::aio::ConnectionManager) -> Router {
    use axum::{middleware as axum_middleware, routing::get};
    use tower_http::cors::{Any, CorsLayer};
    use crate::middleware::{auth::auth_middleware, rate_limit::rate_limit_middleware};

    let app_state = AppState {
        db,
        redis,
        config: config.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let protected_routes = Router::new()
        .nest("/api/v1/auth", routes::auth::protected_router())
        .nest("/api/v1/users", routes::users::router())
        .nest("/api/v1/review", routes::review::router())
        .nest("/api/v1/face-search", routes::face_search::router())
        .nest("/api/v1/publish", routes::publish::router())
        .nest("/api/v1/alerts", routes::alerts::protected_router())
        .nest("/api/v1/map", routes::map::protected_router())
        .nest("/api/v1/stories", routes::stories::protected_router())
        .nest("/api/v1/biz", routes::business::router())
        .nest("/api/v1/admin", routes::admin::router())
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    Router::new()
        .route("/health", get(routes::health::health_check))
        .nest("/api/v1/auth", routes::auth::router())
        .nest("/api/v1/reports", routes::reports::public_router())
        .nest("/api/v1/map", routes::map::public_router())
        .nest("/api/v1/alerts", routes::alerts::public_router())
        .nest("/api/v1/stories", routes::stories::public_router())
        .nest("/api/v1/items", routes::items::public_router())
        .merge(protected_routes)
        .with_state(app_state.clone())
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit_middleware,
        ))
        .layer(cors)
}
