use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::Config;

#[derive(Debug, Deserialize)]
pub struct MeetingsQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MeetingsResponse {
    pub success: bool,
    pub meetings: Vec<Value>,
    pub total: u32,
    pub cached: bool,
}

/// Create router for meetings endpoints
pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/meetings", get(get_meetings))
}

/// GET /api/meetings - Proxy to Fathom API with caching
pub async fn get_meetings(
    axum::extract::State(app_state): axum::extract::State<crate::api::AppState>,
    Query(query): Query<MeetingsQuery>,
    headers: HeaderMap,
) -> Result<Json<MeetingsResponse>, StatusCode> {
    info!("Fetching meetings with query: {:?}", query);

    // Extract authorization token from headers
    let auth_token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");

    if auth_token.is_empty() {
        warn!("No authorization token provided for meetings request");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // TODO: Implement actual Fathom API integration
    // For now, return mock data
    let mock_meetings = vec![
        serde_json::json!({
            "id": "meeting_1",
            "title": "Team Standup",
            "start_time": "2023-12-01T10:00:00Z",
            "duration": 1800,
            "participants": ["user1", "user2", "user3"]
        }),
        serde_json::json!({
            "id": "meeting_2", 
            "title": "Sprint Planning",
            "start_time": "2023-12-01T14:00:00Z",
            "duration": 3600,
            "participants": ["user1", "user4", "user5"]
        }),
    ];

    // TODO: Implement caching to user PocketBase
    // For now, indicate data is not cached
    Ok(Json(MeetingsResponse {
        success: true,
        meetings: mock_meetings,
        total: 2,
        cached: false,
    }))
}

/// Cache meetings data to user's PocketBase instance
async fn cache_meetings_to_pb(
    user_id: &str,
    meetings: &[Value],
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement caching logic
    // 1. Get user's PocketBase instance URL
    // 2. Store meetings data in user's PB database
    // 3. Set appropriate TTL/expiration
    info!("Caching {} meetings for user {}", meetings.len(), user_id);
    Ok(())
}

/// Fetch meetings from user's PocketBase cache
async fn get_cached_meetings(
    user_id: &str,
    config: &Config,
) -> Result<Option<Vec<Value>>, Box<dyn std::error::Error>> {
    // TODO: Implement cache retrieval logic
    // 1. Get user's PocketBase instance URL
    // 2. Query cached meetings data
    // 3. Check if cache is still valid
    info!("Checking cache for user {}", user_id);
    Ok(None)
}
