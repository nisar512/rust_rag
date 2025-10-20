use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::queries::{
    create_chat, create_conversation, create_session, get_chat, get_session,
    list_conversations_by_chat, list_last_conversations_by_chat, update_conversation_response,
};
use crate::services::embedding::EmbeddingService;
use crate::services::gemini::GeminiService;
use crate::utils::config::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub chatbot_id: String,
    pub query: String,
    pub session_id: Option<String>,
    pub chat_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub success: bool,
    pub message: String,
    pub data: ChatData,
}

#[derive(Debug, Serialize)]
pub struct ChatData {
    pub session_id: String,
    pub chat_id: String,
    pub conversation_id: String,
    pub user_query: String,
    pub bot_response: String,
    pub context_used: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionRequest {
    // No fields needed - session is created automatically
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub success: bool,
    pub message: String,
    pub data: SessionData,
}

#[derive(Debug, Serialize)]
pub struct SessionData {
    pub session_id: String,
    pub created_at: String,
}

// Create a new session
pub async fn create_session_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Creating new chat session");

    match create_session(&app_state.db).await {
        Ok(session) => {
            tracing::info!("✅ Session created successfully: {}", session.id);
            Ok(Json(json!({
                "success": true,
                "message": "Session created successfully",
                "data": {
                    "session_id": session.id,
                    "created_at": session.created_at.to_rfc3339()
                }
            })))
        }
        Err(e) => {
            tracing::error!("❌ Failed to create session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Main chat endpoint
pub async fn chat_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Processing chat request: {}", payload.query);

    // Parse chatbot_id
    let chatbot_id = Uuid::parse_str(&payload.chatbot_id).map_err(|e| {
        tracing::error!("Invalid chatbot_id format: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Handle session_id - create new if not provided
    let session_id = match payload.session_id {
        Some(session_id_str) => {
            let session_uuid = Uuid::parse_str(&session_id_str).map_err(|e| {
                tracing::error!("Invalid session_id format: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            
            // Verify session exists
            match get_session(&app_state.db, session_uuid).await {
                Ok(Some(_)) => session_uuid,
                Ok(None) => {
                    tracing::error!("Session not found: {}", session_uuid);
                    return Err(StatusCode::NOT_FOUND);
                }
                Err(e) => {
                    tracing::error!("Failed to get session: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        None => {
            // Create new session
            match create_session(&app_state.db).await {
                Ok(session) => {
                    tracing::info!("Created new session: {}", session.id);
                    session.id
                }
                Err(e) => {
                    tracing::error!("Failed to create session: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
    };

    // Handle chat_id - create new if not provided
    let chat_id = match payload.chat_id {
        Some(chat_id_str) => {
            let chat_uuid = Uuid::parse_str(&chat_id_str).map_err(|e| {
                tracing::error!("Invalid chat_id format: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            
            // Verify chat exists
            match get_chat(&app_state.db, chat_uuid).await {
                Ok(Some(_)) => chat_uuid,
                Ok(None) => {
                    tracing::error!("Chat not found: {}", chat_uuid);
                    return Err(StatusCode::NOT_FOUND);
                }
                Err(e) => {
                    tracing::error!("Failed to get chat: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        None => {
            // Create new chat
            match create_chat(&app_state.db, session_id, "New Chat".to_string()).await {
                Ok(chat) => {
                    tracing::info!("Created new chat: {}", chat.id);
                    chat.id
                }
                Err(e) => {
                    tracing::error!("Failed to create chat: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
    };

    // Create embedding service
    let embedding_service = EmbeddingService::new(app_state.elasticsearch.clone()).map_err(|e| {
        tracing::error!("Failed to create embedding service: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create collection name for this chatbot
    let collection_name = format!("chatbot_{}", chatbot_id);

    // Search for similar embeddings to get context
    let search_results = embedding_service.search_similar(&collection_name, &payload.query, 5).await.map_err(|e| {
        tracing::error!("Failed to search embeddings: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Found {} similar results for query", search_results.len());

    // Prepare context from search results
    let context: String = search_results
        .iter()
        .map(|result| format!("Document: {}\nContent: {}", result.file_path, result.text))
        .collect::<Vec<_>>()
        .join("\n\n");

    // Get conversation history for context (last 5 messages only)
    let conversations = list_last_conversations_by_chat(&app_state.db, chat_id, 5).await.map_err(|e| {
        tracing::error!("Failed to get conversation history: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Build conversation history context (limited to last 5 messages for efficiency)
    let conversation_history: String = conversations
        .iter()
        .map(|conv| {
            format!(
                "User: {}\nBot: {}",
                conv.user_query,
                conv.bot_response.as_ref().unwrap_or(&"".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Create conversation record
    let conversation = create_conversation(
        &app_state.db,
        session_id,
        chat_id,
        payload.query.clone(),
    ).await.map_err(|e| {
        tracing::error!("Failed to create conversation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate response using Gemini
    let gemini_service = GeminiService::new().map_err(|e| {
        tracing::error!("Failed to create Gemini service: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Combine context and conversation history
    let full_context = if !conversation_history.is_empty() {
        format!("Previous conversation:\n{}\n\nRelevant documents:\n{}", conversation_history, context)
    } else {
        format!("Relevant documents:\n{}", context)
    };

    let bot_response = gemini_service.generate_response(&payload.query, &full_context).await.map_err(|e| {
        tracing::error!("Failed to generate response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update conversation with bot response
    let updated_conversation = update_conversation_response(
        &app_state.db,
        conversation.id,
        bot_response.clone(),
    ).await.map_err(|e| {
        tracing::error!("Failed to update conversation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Prepare context used for response
    let context_used: Vec<String> = search_results
        .iter()
        .map(|result| result.file_path.clone())
        .collect();

    tracing::info!("✅ Chat request processed successfully");

    Ok(Json(json!({
        "success": true,
        "message": "Chat request processed successfully",
        "data": {
            "session_id": session_id,
            "chat_id": chat_id,
            "conversation_id": updated_conversation.id,
            "user_query": payload.query,
            "bot_response": bot_response,
            "context_used": context_used
        }
    })))
}

// Get conversation history for a chat
pub async fn get_chat_history_handler(
    State(app_state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let chat_id_str = params.get("chat_id").ok_or(StatusCode::BAD_REQUEST)?;
    let chat_id = Uuid::parse_str(chat_id_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    tracing::info!("Getting chat history for chat: {}", chat_id);

    match list_conversations_by_chat(&app_state.db, chat_id).await {
        Ok(conversations) => {
            let conversation_responses: Vec<Value> = conversations
                .into_iter()
                .map(|conv| {
                    json!({
                        "id": conv.id,
                        "sequence_number": conv.sequence_number,
                        "user_query": conv.user_query,
                        "bot_response": conv.bot_response,
                        "created_at": conv.created_at.to_rfc3339()
                    })
                })
                .collect();

            tracing::info!("✅ Retrieved {} conversations", conversation_responses.len());
            Ok(Json(json!({
                "success": true,
                "message": "Chat history retrieved successfully",
                "data": {
                    "chat_id": chat_id,
                    "conversations": conversation_responses,
                    "count": conversation_responses.len()
                }
            })))
        }
        Err(e) => {
            tracing::error!("❌ Failed to get chat history: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Health check for chat service
pub async fn chat_health_handler() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Chat service is running",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Create the router for chat routes
pub fn create_chat_router() -> Router<AppState> {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/chat/session", post(create_session_handler))
        .route("/chat/history", get(get_chat_history_handler))
        .route("/chat/health", get(chat_health_handler))
}
