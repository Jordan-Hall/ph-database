use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{ApiError, ApiResult},
    services::surrealdb_features::{FullTextSearchService, GraphQueryService, SearchResult},
    AppState,
};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_report))
        .route("/search", get(search_reports))
        .route("/:id", get(get_report))
        .route("/:id/connections", get(get_report_connections))
        .route("/:id/evidence", post(upload_evidence))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: i32,
}

fn default_limit() -> i32 {
    20
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
    total: usize,
}

/// Full-text search across reports using SurrealDB native search
async fn search_reports(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> ApiResult<Json<SearchResponse>> {
    let search_service = FullTextSearchService::new(state.db.clone());

    let results = search_service
        .search_reports(params.q.clone(), params.limit)
        .await?;

    let total = results.len();

    tracing::info!(
        "Full-text search for '{}' returned {} results",
        params.q,
        total
    );

    Ok(Json(SearchResponse { results, total }))
}

/// Get report connections using graph queries
async fn get_report_connections(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let graph_service = GraphQueryService::new(state.db.clone());

    let connections = graph_service
        .find_connected_reports(id)
        .await?;

    Ok(Json(serde_json::to_value(connections).unwrap()))
}

async fn create_report() -> &'static str {
    "TODO: Implement report creation with RLAC"
}

async fn get_report() -> &'static str {
    "TODO: Implement report retrieval with RLAC"
}

async fn upload_evidence() -> &'static str {
    "TODO: Implement evidence upload"
}
