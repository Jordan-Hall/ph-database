/// SurrealDB native features implementation
/// Leverages built-in ML, full-text search, and graph queries
use crate::{
    db::Database,
    error::{ApiError, ApiResult},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Face search service using SurrealDB ML
pub struct FaceSearchService {
    db: Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FaceSearchResult {
    pub id: String,
    pub confidence: f64,
    pub report_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl FaceSearchService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Search for faces using SurrealDB ML function (ephemeral query)
    /// The query image is never stored, only processed in-memory
    pub async fn search_faces(
        &self,
        query_image_base64: String,
        actor_id: String,
    ) -> ApiResult<Vec<FaceSearchResult>> {
        let results: Vec<FaceSearchResult> = self
            .db
            .query_with_params(
                "RETURN fn::search_faces($query_image, $actor_id)",
                json!({
                    "query_image": query_image_base64,
                    "actor_id": actor_id,
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Face search failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("Face search failed"))
            })?;

        Ok(results)
    }

    /// Add a face to the dataset (for convicted individuals only)
    pub async fn add_face_to_dataset(
        &self,
        image_base64: String,
        conviction_record_id: String,
        consent_status: String,
    ) -> ApiResult<String> {
        let result: Vec<serde_json::Value> = self
            .db
            .query_with_params(
                r#"
                LET $embedding = ml::embedding::compute('face_embedding_model', $image);
                CREATE face_dataset_item CONTENT {
                    embedding: $embedding,
                    conviction_record: $conviction_record_id,
                    consent_status: $consent_status,
                    added_at: time::now(),
                    source: 'court_records'
                };
                "#,
                json!({
                    "image": image_base64,
                    "conviction_record_id": conviction_record_id,
                    "consent_status": consent_status,
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to add face to dataset: {}", e);
                ApiError::Internal(anyhow::anyhow!("Failed to add face"))
            })?;

        Ok(result
            .first()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string())
    }
}

/// Full-text search service using SurrealDB native search
pub struct FullTextSearchService {
    db: Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub relevance: f64,
    pub highlight: Option<String>,
    pub record_type: String,
}

impl FullTextSearchService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Search reports using SurrealDB full-text search with BM25 ranking
    pub async fn search_reports(&self, query: String, limit: i32) -> ApiResult<Vec<SearchResult>> {
        let results: Vec<SearchResult> = self
            .db
            .query_with_params(
                r#"
                SELECT *,
                    search::score(1) * 2 AS relevance,
                    search::highlight('<mark>', '</mark>', 1) AS highlight
                FROM report
                WHERE title @1@ $query OR description @2@ $query
                ORDER BY relevance DESC
                LIMIT $limit
                "#,
                json!({
                    "query": query,
                    "limit": limit,
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Full-text search failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("Search failed"))
            })?;

        Ok(results)
    }

    /// Search across multiple content types
    pub async fn search_all(
        &self,
        query: String,
        _content_types: Vec<String>,
        limit: i32,
    ) -> ApiResult<Vec<SearchResult>> {
        // Union search across reports, survivor stories, and conviction records
        let results: Vec<SearchResult> = self
            .db
            .query_with_params(
                r#"
                RETURN fn::search_reports($query, $limit)
                "#,
                json!({
                    "query": query,
                    "limit": limit,
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Multi-content search failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("Search failed"))
            })?;

        Ok(results)
    }
}

/// Graph query service for relationship analysis
pub struct GraphQueryService {
    db: Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub report_id: String,
    pub connected_reports: Vec<String>,
    pub connection_strength: i32,
    pub common_people: Vec<String>,
}

impl GraphQueryService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Find reports connected through shared conviction records (people)
    pub async fn find_connected_reports(&self, report_id: String) -> ApiResult<ConnectionResult> {
        let results: Vec<ConnectionResult> = self
            .db
            .query_with_params(
                r#"
                RETURN fn::find_connected_reports($report_id)
                "#,
                json!({
                    "report_id": report_id,
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Graph query failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("Graph query failed"))
            })?;

        results
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::NotFound("Report not found".to_string()))
    }

    /// Find all reports involving a specific person
    pub async fn find_reports_by_person(
        &self,
        conviction_record_id: String,
    ) -> ApiResult<Vec<String>> {
        let results: Vec<serde_json::Value> = self
            .db
            .query_with_params(
                r#"
                SELECT <-involves_person<-report AS reports
                FROM $conviction_id
                "#,
                json!({
                    "conviction_id": conviction_record_id,
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Person reports query failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("Query failed"))
            })?;

        // Extract report IDs from results
        let report_ids: Vec<String> = results
            .into_iter()
            .filter_map(|v| {
                v.get("reports")
                    .and_then(|r| r.as_array())
                    .map(|arr| arr.to_vec())
            })
            .flatten()
            .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
            .collect();

        Ok(report_ids)
    }

    /// Analyze network of connections for pattern detection
    pub async fn analyze_connection_network(
        &self,
        center_report_id: String,
        depth: i32,
    ) -> ApiResult<serde_json::Value> {
        let results: Vec<serde_json::Value> = self
            .db
            .query_with_params(
                r#"
                -- Traverse the graph from a central report
                SELECT *,
                    ->involves_person->conviction_record<-involves_person<-report AS connections
                FROM $report_id
                "#,
                json!({
                    "report_id": center_report_id,
                    "depth": depth,
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Network analysis failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("Network analysis failed"))
            })?;

        Ok(results.into_iter().next().unwrap_or(json!({})))
    }
}

/// Alert management service with TTL
pub struct AlertService {
    db: Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alert {
    pub id: Option<String>,
    pub alert_type: String,
    pub title: String,
    pub description: String,
    pub expires_at: String,
}

impl AlertService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a missing person alert with TTL
    pub async fn create_missing_person_alert(
        &self,
        title: String,
        description: String,
        ttl_hours: i32,
    ) -> ApiResult<String> {
        let results: Vec<serde_json::Value> = self
            .db
            .query_with_params(
                r#"
                CREATE missing_person_alert CONTENT {
                    title: $title,
                    description: $description,
                    status: 'active',
                    created_at: time::now(),
                    expires_at: time::now() + $ttl,
                    priority: 'high'
                }
                "#,
                json!({
                    "title": title,
                    "description": description,
                    "ttl": format!("{}h", ttl_hours),
                }),
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to create alert: {}", e);
                ApiError::Internal(anyhow::anyhow!("Failed to create alert"))
            })?;

        Ok(results
            .first()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string())
    }

    /// Expire old alerts using SurrealDB function
    pub async fn expire_old_alerts(&self) -> ApiResult<i32> {
        let results: Vec<serde_json::Value> = self
            .db
            .query("RETURN fn::expire_old_alerts()")
            .await
            .map_err(|e| {
                tracing::error!("Failed to expire alerts: {}", e);
                ApiError::Internal(anyhow::anyhow!("Failed to expire alerts"))
            })?;

        let count = results
            .first()
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        Ok(count)
    }
}
