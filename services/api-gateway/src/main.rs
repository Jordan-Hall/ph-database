mod auth;
mod config;
mod db;
mod error;
mod middleware;
mod models;
mod routes;
mod services;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    db::Database,
    middleware::{auth::auth_middleware, rate_limit::rate_limit_middleware},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize Prometheus metrics exporter
    let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");
    tracing::info!("Metrics exporter initialized");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    // Initialize database connection
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connected");

    // Initialize Redis for rate limiting and sessions
    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis_conn = redis_client.get_connection_manager().await?;
    tracing::info!("Redis connected");

    // Build application state
    let app_state = AppState {
        db,
        redis: redis_conn,
        config: config.clone(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build protected routes that require authentication
    let protected_routes = Router::new()
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

    // Build our application router
    let app = Router::new()
        // Health check
        .route("/health", get(routes::health::health_check))
        // Metrics endpoint for Prometheus
        .route("/metrics", get(move || async move {
            metrics_handle.render()
        }))

        // Public routes (no auth required)
        .nest("/api/v1/auth", routes::auth::router())
        .nest("/api/v1/reports", routes::reports::public_router())
        .nest("/api/v1/map", routes::map::public_router())
        .nest("/api/v1/alerts", routes::alerts::public_router())
        .nest("/api/v1/stories", routes::stories::public_router())
        .nest("/api/v1/items", routes::items::public_router())

        // Merge protected routes
        .merge(protected_routes)

        // Apply state first
        .with_state(app_state.clone())

        // Then apply stateful middleware
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit_middleware,
        ))

        // Finally apply stateless middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("API Gateway listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: redis::aio::ConnectionManager,
    pub config: Config,
}
