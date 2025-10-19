use crate::db::models::*;
use crate::errors::AppResult;
use sqlx::PgPool;
use uuid::Uuid;

// Session queries
pub async fn create_session(pool: &PgPool) -> AppResult<Session> {
    let session = sqlx::query_as::<_, Session>(
        "INSERT INTO sessions DEFAULT VALUES RETURNING *"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(session)
}

pub async fn get_session(pool: &PgPool, session_id: Uuid) -> AppResult<Option<Session>> {
    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE id = $1 AND status = 'active'"
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(session)
}

pub async fn list_sessions(pool: &PgPool) -> AppResult<Vec<Session>> {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE status = 'active' ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(sessions)
}

// Chat queries
pub async fn create_chat(pool: &PgPool, session_id: Uuid, title: String) -> AppResult<Chat> {
    let chat = sqlx::query_as::<_, Chat>(
        "INSERT INTO chats (session_id, title) VALUES ($1, $2) RETURNING *"
    )
    .bind(session_id)
    .bind(title)
    .fetch_one(pool)
    .await?;
    
    Ok(chat)
}

pub async fn get_chat(pool: &PgPool, chat_id: Uuid) -> AppResult<Option<Chat>> {
    let chat = sqlx::query_as::<_, Chat>(
        "SELECT * FROM chats WHERE id = $1 AND status = 'active'"
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(chat)
}

pub async fn list_chats_by_session(pool: &PgPool, session_id: Uuid) -> AppResult<Vec<Chat>> {
    let chats = sqlx::query_as::<_, Chat>(
        "SELECT * FROM chats WHERE session_id = $1 AND status = 'active' ORDER BY created_at ASC"
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    
    Ok(chats)
}

// Conversation queries
pub async fn create_conversation(
    pool: &PgPool,
    session_id: Uuid,
    chat_id: Uuid,
    user_query: String,
) -> AppResult<Conversation> {
    // Get the next sequence number for this chat
    let next_sequence: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM conversations WHERE chat_id = $1"
    )
    .bind(chat_id)
    .fetch_one(pool)
    .await?;

    let conversation = sqlx::query_as::<_, Conversation>(
        "INSERT INTO conversations (session_id, chat_id, sequence_number, user_query) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(session_id)
    .bind(chat_id)
    .bind(next_sequence)
    .bind(user_query)
    .fetch_one(pool)
    .await?;
    
    Ok(conversation)
}

pub async fn update_conversation_response(
    pool: &PgPool,
    conversation_id: Uuid,
    bot_response: String,
) -> AppResult<Conversation> {
    let conversation = sqlx::query_as::<_, Conversation>(
        "UPDATE conversations SET bot_response = $1 WHERE id = $2 AND status = 'active' RETURNING *"
    )
    .bind(bot_response)
    .bind(conversation_id)
    .fetch_one(pool)
    .await?;
    
    Ok(conversation)
}

pub async fn get_conversation(pool: &PgPool, conversation_id: Uuid) -> AppResult<Option<Conversation>> {
    let conversation = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE id = $1 AND status = 'active'"
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(conversation)
}

pub async fn list_conversations_by_chat(pool: &PgPool, chat_id: Uuid) -> AppResult<Vec<Conversation>> {
    let conversations = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE chat_id = $1 AND status = 'active' ORDER BY sequence_number ASC"
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await?;
    
    Ok(conversations)
}

pub async fn list_conversations_by_session(pool: &PgPool, session_id: Uuid) -> AppResult<Vec<Conversation>> {
    let conversations = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE session_id = $1 AND status = 'active' ORDER BY created_at ASC"
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    
    Ok(conversations)
}

// ChatBot queries
pub async fn create_chat_bot(pool: &PgPool, name: String) -> AppResult<ChatBot> {
    let chat_bot = sqlx::query_as::<_, ChatBot>(
        "INSERT INTO chat_bot (name) VALUES ($1) RETURNING *"
    )
    .bind(name)
    .fetch_one(pool)
    .await?;
    
    Ok(chat_bot)
}

pub async fn get_chat_bot(pool: &PgPool, chat_bot_id: Uuid) -> AppResult<Option<ChatBot>> {
    let chat_bot = sqlx::query_as::<_, ChatBot>(
        "SELECT * FROM chat_bot WHERE id = $1 AND status = 'active'"
    )
    .bind(chat_bot_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(chat_bot)
}

pub async fn list_chat_bots(pool: &PgPool) -> AppResult<Vec<ChatBot>> {
    let chat_bots = sqlx::query_as::<_, ChatBot>(
        "SELECT * FROM chat_bot WHERE status = 'active' ORDER BY created_at ASC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(chat_bots)
}

pub async fn update_chat_bot(pool: &PgPool, chat_bot_id: Uuid, name: String) -> AppResult<ChatBot> {
    let chat_bot = sqlx::query_as::<_, ChatBot>(
        "UPDATE chat_bot SET name = $1 WHERE id = $2 AND status = 'active' RETURNING *"
    )
    .bind(name)
    .bind(chat_bot_id)
    .fetch_one(pool)
    .await?;
    
    Ok(chat_bot)
}

pub async fn delete_chat_bot(pool: &PgPool, chat_bot_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE chat_bot SET status = 'deleted' WHERE id = $1")
        .bind(chat_bot_id)
        .execute(pool)
        .await?;
    
    Ok(())
}