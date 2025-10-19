use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub mod models;
pub mod queries;

pub async fn init_db() -> Result<PgPool> {
    let database_url = env::var("DATABASE_URL")
        .expect(" DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to Postgres successfully");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    tracing::info!("Running database migrations...");
    
    // Create tables first
    sqlx::query("CREATE TABLE IF NOT EXISTS sessions (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'deleted'))
    )").execute(pool).await?;
    
    sqlx::query("CREATE TABLE IF NOT EXISTS chats (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
        title VARCHAR(255) NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'deleted'))
    )").execute(pool).await?;
    
    sqlx::query("CREATE TABLE IF NOT EXISTS conversations (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
        chat_id UUID NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
        sequence_number INTEGER NOT NULL,
        user_query TEXT NOT NULL,
        bot_response TEXT,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'deleted')),
        UNIQUE(chat_id, sequence_number)
    )").execute(pool).await?;
    
    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_chats_session_id ON chats(session_id)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_conversations_session_id ON conversations(session_id)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_conversations_chat_id ON conversations(chat_id)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_conversations_sequence ON conversations(chat_id, sequence_number)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_conversations_created_at ON conversations(created_at)")
        .execute(pool).await?;
    
    // Create function and triggers
    sqlx::query("CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql'").execute(pool).await?;
    
    sqlx::query("DROP TRIGGER IF EXISTS update_sessions_updated_at ON sessions")
        .execute(pool).await?;
    sqlx::query("CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE ON sessions
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()")
        .execute(pool).await?;
    
    sqlx::query("DROP TRIGGER IF EXISTS update_chats_updated_at ON chats")
        .execute(pool).await?;
    sqlx::query("CREATE TRIGGER update_chats_updated_at BEFORE UPDATE ON chats
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()")
        .execute(pool).await?;
    
    sqlx::query("DROP TRIGGER IF EXISTS update_conversations_updated_at ON conversations")
        .execute(pool).await?;
    sqlx::query("CREATE TRIGGER update_conversations_updated_at BEFORE UPDATE ON conversations
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()")
        .execute(pool).await?;
    
    tracing::info!("✅ Database migrations completed successfully");
    Ok(())
}
