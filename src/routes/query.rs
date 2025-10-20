use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::services::embedding::EmbeddingService;
use crate::services::elasticsearch::SearchResult;
use crate::utils::config::AppState;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub chatbot_id: String,
    pub query: String,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub success: bool,
    pub message: String,
    pub data: QueryData,
}

#[derive(Debug, Serialize)]
pub struct QueryData {
    pub chatbot_id: String,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: usize,
}

// Query endpoint for similarity search
pub async fn query_handler(
    State(app_state): State<AppState>,
    Query(params): Query<QueryRequest>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Processing query: {}", params.query);

    // Parse chatbot_id
    let chatbot_id = Uuid::parse_str(&params.chatbot_id).map_err(|e| {
        tracing::error!("Invalid chatbot_id format: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let limit = params.limit.unwrap_or(5);

    // Create embedding service
    let embedding_service = EmbeddingService::new(app_state.elasticsearch.clone()).map_err(|e| {
        tracing::error!("Failed to create embedding service: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create collection name for this chatbot
    let collection_name = format!("chatbot_{}", chatbot_id);

    // Search for similar embeddings
    let search_results = embedding_service.search_similar(&collection_name, &params.query, limit).await.map_err(|e| {
        tracing::error!("Failed to search embeddings: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Found {} similar results for query", search_results.len());

    Ok(Json(json!({
        "success": true,
        "message": "Query processed successfully",
        "data": {
            "chatbot_id": params.chatbot_id,
            "query": params.query,
            "results": search_results,
            "total_results": search_results.len()
        }
    })))
}

// Health check for query service
pub async fn query_health_handler() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Query service is running",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Create the router for query routes
pub fn create_query_router() -> Router<AppState> {
    Router::new()
        .route("/query", get(query_handler))
        .route("/query/health", get(query_health_handler))
}
