use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        ReviewStoryRequest, StoryQueryParams, StoryStatus, SubmitStoryRequest,
        SurvivorStory,
    },
    AppState,
};

// ============================================================================
// PUBLIC ROUTES (no authentication required)
// ============================================================================

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_published_stories))
        .route("/:id", get(get_story))
        .route("/submit", post(submit_story))
}

/// GET /api/v1/stories
/// List all published stories (public access)
async fn get_published_stories(
    State(state): State<AppState>,
    Query(params): Query<StoryQueryParams>,
) -> ApiResult<Json<Vec<SurvivorStory>>> {
    let query = format!(
        "SELECT * FROM survivor_story WHERE status = 'published' AND visibility = 'public'
         ORDER BY published_at DESC LIMIT {}",
        params.limit.min(100)
    );

    let stories: Vec<SurvivorStory> = state
        .db
        .query(&query)
        .await
?;

    Ok(Json(stories))
}

/// GET /api/v1/stories/:id
/// Get a single story (public if published, otherwise requires auth via RLAC)
async fn get_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<SurvivorStory>> {
    let query = format!("SELECT * FROM survivor_story WHERE id = survivor_story:{}", id);

    let mut stories: Vec<SurvivorStory> = state
        .db
        .query(&query)
        .await
?;

    let story = stories
        .pop()
        .ok_or_else(|| ApiError::NotFound("Story not found".to_string()))?;

    // RLAC will handle permission checks at database level
    // Public can only see published+public stories
    // Reviewers/admins can see all stories
    // Submitters can see their own stories

    Ok(Json(story))
}

/// POST /api/v1/stories/submit
/// Submit a survivor story (can be authenticated or anonymous)
async fn submit_story(
    State(state): State<AppState>,
    Json(payload): Json<SubmitStoryRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate input
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Check consent
    if !payload.consent_given {
        return Err(ApiError::Validation(
            "Consent must be given to submit a story".to_string(),
        ));
    }

    // Create story record
    let query = format!(
        r#"CREATE survivor_story CONTENT {{
            title: '{}',
            content: '{}',
            author_pseudonym: {},
            consent_given: true,
            consent_date: time::now(),
            consent_details: {},
            status: 'pending',
            visibility: 'private',
            trigger_warning: {},
            submitted_by: $auth.id,
            created_at: time::now(),
            updated_at: time::now()
        }}"#,
        payload.title.replace("'", "\\'"),
        payload.content.replace("'", "\\'"),
        payload
            .author_pseudonym
            .as_ref()
            .map(|p| format!("'{}'", p.replace("'", "\\'")))
            .unwrap_or_else(|| "NONE".to_string()),
        payload
            .consent_details
            .as_ref()
            .map(|d| format!("'{}'", d.replace("'", "\\'")))
            .unwrap_or_else(|| "NONE".to_string()),
        payload
            .trigger_warning
            .as_ref()
            .map(|t| format!("'{}'", t.replace("'", "\\'")))
            .unwrap_or_else(|| "NONE".to_string())
    );

    let mut stories: Vec<SurvivorStory> = state
        .db
        .query(&query)
        .await
?;

    let story = stories
        .pop()
        .ok_or_else(|| ApiError::BadRequest("Failed to create story".to_string()))?;

    Ok(Json(json!({
        "id": story.id,
        "status": story.status,
        "message": "Your story has been submitted for review. Thank you for sharing.",
        "created_at": story.created_at
    })))
}

// ============================================================================
// PROTECTED ROUTES (authentication required)
// ============================================================================

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/review", get(get_stories_for_review))
        .route("/:id/review", post(review_story))
        .route("/:id/publish", post(publish_story))
        .route("/:id/withdraw", post(withdraw_story))
}

/// GET /api/v1/stories/review
/// Get stories pending review (reviewer/admin only)
async fn get_stories_for_review(
    State(state): State<AppState>,
    Query(params): Query<StoryQueryParams>,
) -> ApiResult<Json<Vec<SurvivorStory>>> {
    // Filter by status if provided, otherwise show pending + reviewing
    let status_filter = if let Some(status) = params.status {
        format!("status = '{}'", status)
    } else {
        "status IN ['pending', 'reviewing']".to_string()
    };

    let query = format!(
        "SELECT * FROM survivor_story WHERE {}
         ORDER BY created_at ASC LIMIT {}",
        status_filter,
        params.limit.min(100)
    );

    let stories: Vec<SurvivorStory> = state
        .db
        .query(&query)
        .await
?;

    // RLAC will enforce that only reviewers/admins can access this
    Ok(Json(stories))
}

/// POST /api/v1/stories/:id/review
/// Review a story (approve or reject)
async fn review_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ReviewStoryRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate status transition
    match payload.status {
        StoryStatus::Approved | StoryStatus::Rejected => {}
        _ => {
            return Err(ApiError::Validation(
                "Status must be 'approved' or 'rejected'".to_string(),
            ))
        }
    }

    let query = format!(
        r#"UPDATE survivor_story:{} SET
            status = '{}',
            reviewed_by = $auth.id,
            reviewed_at = time::now(),
            review_notes = {},
            updated_at = time::now()"#,
        id,
        match payload.status {
            StoryStatus::Approved => "approved",
            StoryStatus::Rejected => "rejected",
            _ => unreachable!(),
        },
        payload
            .review_notes
            .as_ref()
            .map(|n| format!("'{}'", n.replace("'", "\\'")))
            .unwrap_or_else(|| "NONE".to_string())
    );

    let mut stories: Vec<SurvivorStory> = state
        .db
        .query(&query)
        .await
?;

    let story = stories
        .pop()
        .ok_or_else(|| ApiError::NotFound("Story not found or not authorized".to_string()))?;

    Ok(Json(json!({
        "id": story.id,
        "status": story.status,
        "reviewed_by": story.reviewed_by,
        "reviewed_at": story.reviewed_at,
        "message": format!("Story has been {}", match payload.status {
            StoryStatus::Approved => "approved",
            StoryStatus::Rejected => "rejected",
            _ => "updated",
        })
    })))
}

/// POST /api/v1/stories/:id/publish
/// Publish an approved story
async fn publish_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // First check if story is approved
    let check_query = format!("SELECT * FROM survivor_story:{}", id);

    let mut stories: Vec<SurvivorStory> = state
        .db
        .query(&check_query)
        .await
?;

    let story = stories
        .pop()
        .ok_or_else(|| ApiError::NotFound("Story not found or not authorized".to_string()))?;

    if story.status != StoryStatus::Approved {
        return Err(ApiError::Validation(
            "Only approved stories can be published".to_string(),
        ));
    }

    // Publish the story
    let update_query = format!(
        r#"UPDATE survivor_story:{} SET
            status = 'published',
            visibility = 'public',
            published_at = time::now(),
            updated_at = time::now()"#,
        id
    );

    let mut updated: Vec<SurvivorStory> = state
        .db
        .query(&update_query)
        .await
?;

    let published_story = updated
        .pop()
        .ok_or_else(|| ApiError::BadRequest("Failed to publish story".to_string()))?;

    Ok(Json(json!({
        "id": published_story.id,
        "status": published_story.status,
        "visibility": published_story.visibility,
        "published_at": published_story.published_at,
        "message": "Story has been published successfully"
    })))
}

/// POST /api/v1/stories/:id/withdraw
/// Withdraw a published story (make it private)
async fn withdraw_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = format!(
        r#"UPDATE survivor_story:{} SET
            visibility = 'private',
            updated_at = time::now()"#,
        id
    );

    let mut stories: Vec<SurvivorStory> = state
        .db
        .query(&query)
        .await
?;

    let story = stories
        .pop()
        .ok_or_else(|| ApiError::NotFound("Story not found or not authorized".to_string()))?;

    Ok(Json(json!({
        "id": story.id,
        "visibility": story.visibility,
        "message": "Story has been withdrawn from public view"
    })))
}
