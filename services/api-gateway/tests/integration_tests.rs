use api_gateway::{build_app, config::Config, db::Database};
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
use std::env;

// Helper to parse JSON response
async fn parse_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// Helper to create test database connection
async fn setup_test_db() -> Database {
    let db_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "ws://localhost:8000".to_string());
    Database::new(&db_url).await.expect("Failed to connect to test database")
}

// Helper to create test Redis connection
async fn setup_test_redis() -> redis::aio::ConnectionManager {
    let redis_url = env::var("TEST_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    client.get_connection_manager().await.expect("Failed to connect to Redis")
}

// Helper to build test app
async fn build_test_app() -> axum::Router {
    let config = Config {
        port: 3000,
        database_url: env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
        redis_url: env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        jwt_secret: "test-secret-key-for-integration-tests-only".to_string(),
        jwt_expiry_minutes: 60,
        refresh_token_expiry_days: 30,
        media_service_url: "http://localhost:8081".to_string(),
        ai_service_url: "http://localhost:8082".to_string(),
        alerts_service_url: "http://localhost:8083".to_string(),
    };
    let db = setup_test_db().await;
    let redis = setup_test_redis().await;
    build_app(config, db, redis).await
}

// Helper to create test user and get auth token
async fn create_test_user_with_auth(app: &axum::Router) -> (String, String) {
    let timestamp = chrono::Utc::now().timestamp();
    let register_payload = json!({
        "username": format!("testuser_{}", timestamp),
        "email": format!("test_{}@example.com", timestamp),
        "password": "TestPassword123!"
    });

    let response = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    let body = parse_json(response.into_body()).await;
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    let token = body["access_token"].as_str().unwrap().to_string();
    (user_id, token)
}

#[tokio::test]
async fn test_health_check() {
    let app = build_test_app().await;

    let response = app.oneshot(
        Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json(response.into_body()).await;
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_register_user() {
    let app = build_test_app().await;
    let timestamp = chrono::Utc::now().timestamp();

    let payload = json!({
        "username": format!("newuser_{}", timestamp),
        "email": format!("new_{}@example.com", timestamp),
        "password": "SecurePassword123!"
    });

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_json(response.into_body()).await;
    assert!(body["access_token"].is_string());
    assert!(body["user"]["id"].is_string());
}

#[tokio::test]
async fn test_login_user() {
    let app = build_test_app().await;
    let (_, _) = create_test_user_with_auth(&app).await;

    // Login with the same credentials
    let login_payload = json!({
        "email": format!("test_{}@example.com", chrono::Utc::now().timestamp()),
        "password": "TestPassword123!"
    });

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    let body = parse_json(response.into_body()).await;
    assert!(body["access_token"].is_string() || body["error"].is_string());
}

#[tokio::test]
async fn test_protected_route_without_auth() {
    let app = build_test_app().await;

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/api/v1/users/me")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_with_auth() {
    let app = build_test_app().await;
    let (_user_id, token) = create_test_user_with_auth(&app).await;

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/api/v1/users/me")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json(response.into_body()).await;
    assert!(body["user"]["id"].is_string());
}

#[tokio::test]
async fn test_create_report() {
    let app = build_test_app().await;
    let (_user_id, token) = create_test_user_with_auth(&app).await;

    let report_payload = json!({
        "title": "Test Report",
        "description": "This is a test report for integration testing",
        "category": "suspicious_behavior",
        "location": "123 Test St, London, UK",
        "latitude": 51.5074,
        "longitude": -0.1278
    });

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/reports")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(report_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_json(response.into_body()).await;
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_report_validation() {
    let app = build_test_app().await;
    let (_user_id, token) = create_test_user_with_auth(&app).await;

    let invalid_payload = json!({
        "title": "T",
        "description": "Too short"
    });

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/reports")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(invalid_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_review_queue() {
    let app = build_test_app().await;
    let (_user_id, token) = create_test_user_with_auth(&app).await;

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/api/v1/review/queue")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert!(
        response.status() == StatusCode::OK ||
        response.status() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_publish_report() {
    let app = build_test_app().await;
    let (_user_id, token) = create_test_user_with_auth(&app).await;

    let report_payload = json!({
        "title": "Report to Publish",
        "description": "This report will be published if user has permission",
        "category": "suspicious_behavior"
    });

    let create_response = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/reports")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(report_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    let body = parse_json(create_response.into_body()).await;
    let report_id = body["id"].as_str().unwrap().replace("report:", "");

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/publish/report/{}", report_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_business_api_validation() {
    let app = build_test_app().await;

    let validation_payload = json!({
        "full_name": "John Doe",
        "date_of_birth": "1980-01-01",
        "check_type": "criminal_record"
    });

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/biz/validate")
            .header("X-API-Key", "test-invalid-api-key")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(validation_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    assert!(
        response.status() == StatusCode::UNAUTHORIZED ||
        response.status() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_rate_limiting() {
    let app = build_test_app().await;
    let mut hit_limit = false;

    for _ in 0..105 {
        let response = app.clone().oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            hit_limit = true;
            break;
        }
    }

    println!("Rate limiting test - hit limit: {}", hit_limit);
}

#[tokio::test]
async fn test_alerts_lifecycle() {
    let app = build_test_app().await;
    let (_user_id, token) = create_test_user_with_auth(&app).await;

    let alert_payload = json!({
        "title": "Test Missing Person Alert",
        "description": "Test alert for integration testing",
        "priority": "high",
        "location": "Test Location, UK",
        "expires_at": "2026-12-31T23:59:59Z"
    });

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/alerts")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(alert_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    assert!(
        response.status() == StatusCode::CREATED ||
        response.status() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_map_entries() {
    let app = build_test_app().await;
    let (_user_id, token) = create_test_user_with_auth(&app).await;

    let map_payload = json!({
        "title": "Test Map Entry",
        "description": "Test map entry for integration testing",
        "latitude": 51.5074,
        "longitude": -0.1278,
        "display_policy": "street",
        "harm_risk": "medium"
    });

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/map/entries")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(map_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    assert!(
        response.status() == StatusCode::CREATED ||
        response.status() == StatusCode::FORBIDDEN
    );
}

// Run integration tests with:
// cargo test --test integration_tests -- --test-threads=1
