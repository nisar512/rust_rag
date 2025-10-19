use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chat {
    pub id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub session_id: Uuid,
    pub chat_id: Uuid,
    pub sequence_number: i32,
    pub user_query: String,
    pub bot_response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}

// Request/Response DTOs for API
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    // No fields needed - session is created automatically
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChatRequest {
    pub session_id: Uuid,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub session_id: Uuid,
    pub chat_id: Uuid,
    pub user_query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConversationRequest {
    pub bot_response: String,
}

// Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub chats: Vec<ChatResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub conversations: Vec<ConversationResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub chat_id: Uuid,
    pub sequence_number: i32,
    pub user_query: String,
    pub bot_response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}
