use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};

use crate::db::models::{CreateChatBotRequest, ChatBotResponse};
use crate::db::queries::{create_chat_bot, list_chat_bots};
use crate::utils::config::AppState;

// Create a new chatbot
pub async fn create_chatbot_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateChatBotRequest>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Creating chatbot with name: {}", payload.name);

    match create_chat_bot(&app_state.db, payload.name).await {
        Ok(chatbot) => {
            let response = ChatBotResponse {
                id: chatbot.id,
                name: chatbot.name,
                created_at: chatbot.created_at,
                updated_at: chatbot.updated_at,
                status: chatbot.status,
            };

            tracing::info!("✅ Chatbot created successfully: {}", response.id);
            Ok(Json(json!({
                "success": true,
                "message": "Chatbot created successfully",
                "data": response
            })))
        }
        Err(e) => {
            tracing::error!("❌ Failed to create chatbot: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get all chatbots
pub async fn get_chatbots_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Fetching all chatbots");

    match list_chat_bots(&app_state.db).await {
        Ok(chatbots) => {
            let responses: Vec<ChatBotResponse> = chatbots
                .into_iter()
                .map(|chatbot| ChatBotResponse {
                    id: chatbot.id,
                    name: chatbot.name,
                    created_at: chatbot.created_at,
                    updated_at: chatbot.updated_at,
                    status: chatbot.status,
                })
                .collect();

            tracing::info!("✅ Retrieved {} chatbots", responses.len());
            Ok(Json(json!({
                "success": true,
                "message": "Chatbots retrieved successfully",
                "data": responses,
                "count": responses.len()
            })))
        }
        Err(e) => {
            tracing::error!("❌ Failed to fetch chatbots: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Create the router for chatbot routes
pub fn create_chatbot_router() -> Router<AppState> {
    Router::new()
        .route("/chatbots", post(create_chatbot_handler))
        .route("/chatbots", get(get_chatbots_handler))
}
