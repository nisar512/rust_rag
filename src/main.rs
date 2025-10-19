use axum::{routing::get, Router, response::Json};
use dotenv::dotenv;
use std::{net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde_json::{json, Value};
use qdrant_client::Qdrant;

mod routes;
mod db;
mod utils;
// mod services;
mod errors;

use db::{init_db, run_migrations};
use utils::config::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenv().ok();

    // Setup tracing/logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting RAG Server...");

    // Initialize DB and Qdrant - server will not start if either fails
    tracing::info!("Connecting to database...");
    let pool = init_db().await?;
    tracing::info!("✅ Database connected successfully");
    
    // Run database migrations
    run_migrations(&pool).await?;

    tracing::info!("Connecting to Qdrant...");
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or("http://localhost:6333".to_string());
    
    // Build Qdrant client - try simple approach first
    let qdrant_client = Qdrant::from_url(&qdrant_url).build()?;
    
    // Test Qdrant connection - server will fail to start if this fails
    tracing::info!("Testing Qdrant connection...");
    
    // Try multiple connection methods to handle HTTP/2 issues
    let mut connection_verified = false;
    
    // Method 1: Try health check
    match qdrant_client.health_check().await {
        Ok(_) => {
            tracing::info!("✅ Qdrant health check passed");
            connection_verified = true;
        },
        Err(e) => {
            tracing::warn!("⚠️ Qdrant health check failed: {}", e);
        }
    }
    
    // Method 2: Try collections list if health check failed
    if !connection_verified {
        tracing::info!("Trying alternative connection test...");
        match qdrant_client.list_collections().await {
            Ok(_) => {
                tracing::info!("✅ Qdrant connection verified via collections list");
                connection_verified = true;
            },
            Err(e) => {
                tracing::warn!("⚠️ Collections list also failed: {}", e);
            }
        }
    }
    
    // Method 3: Try simple HTTP request as last resort
    if !connection_verified {
        tracing::info!("Trying direct HTTP connection test...");
        match reqwest::get(&format!("{}/collections", qdrant_url)).await {
            Ok(response) if response.status().is_success() => {
                tracing::info!("✅ Qdrant HTTP connection verified");
                connection_verified = true;
            },
            Ok(response) => {
                tracing::warn!("⚠️ HTTP test returned status: {}", response.status());
            },
            Err(e) => {
                tracing::warn!("⚠️ HTTP test failed: {}", e);
            }
        }
    }
    
    // Final check - fail if no method worked
    if !connection_verified {
        tracing::error!("❌ All Qdrant connection tests failed");
        tracing::error!("Server cannot start without Qdrant connection");
        tracing::error!("Please ensure Qdrant is running on {}", qdrant_url);
        tracing::error!("Try: docker run -p 6333:6333 qdrant/qdrant");
        return Err(anyhow::anyhow!("Qdrant connection failed - all connection methods failed"));
    }
    
    tracing::info!("✅ Qdrant connection verified successfully");

    // Shared application state
    let app_state = AppState {
        db: Arc::new(pool),
        qdrant: Arc::new(qdrant_client),
    };

    // Health check handler
    async fn health_handler() -> Json<Value> {
        Json(json!({
            "status": "ok",
            "message": "RAG Server is running",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    // Define routes
    let app = Router::new()
        .route("/health", get(health_handler))
        .nest("/api", routes::chatbot::create_chatbot_router())
        .with_state(app_state);

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("🌍 Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
