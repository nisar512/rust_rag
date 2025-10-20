use axum::{routing::get, Router, response::Json};
use dotenv::dotenv;
use std::{net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde_json::{json, Value};
use elasticsearch::{Elasticsearch, http::transport::Transport};
use tower_http::cors::{CorsLayer, Any};

mod routes;
mod db;
mod utils;
mod services;
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

    tracing::info!("Connecting to Elasticsearch...");
    let elasticsearch_url = std::env::var("ELASTICSEARCH_URL").unwrap_or("http://localhost:9200".to_string());
    
    // Build Elasticsearch client
    let transport = Transport::single_node(&elasticsearch_url)?;
    let elasticsearch_client = Elasticsearch::new(transport);
    
    // Test Elasticsearch connection - server will fail to start if this fails
    tracing::info!("Testing Elasticsearch connection...");
    
    let mut connection_verified = false;
    
    // Try ping check
    match elasticsearch_client.ping().send().await {
        Ok(response) if response.status_code().is_success() => {
            tracing::info!("✅ Elasticsearch ping successful");
            connection_verified = true;
        },
        Ok(response) => {
            tracing::warn!("⚠️ Elasticsearch ping returned status: {}", response.status_code());
        },
        Err(e) => {
            tracing::warn!("⚠️ Elasticsearch ping failed: {}", e);
        }
    }
    
    // Try alternative connection test if ping failed
    if !connection_verified {
        tracing::info!("Trying alternative connection test...");
        match elasticsearch_client.cat().health().send().await {
            Ok(response) if response.status_code().is_success() => {
                tracing::info!("✅ Elasticsearch cat health successful");
                connection_verified = true;
            },
            Ok(response) => {
                tracing::warn!("⚠️ Elasticsearch cat health returned status: {}", response.status_code());
            },
            Err(e) => {
                tracing::warn!("⚠️ Elasticsearch cat health failed: {}", e);
            }
        }
    }
    
    // Final check - fail if no method worked
    if !connection_verified {
        tracing::error!("❌ All Elasticsearch connection tests failed");
        tracing::error!("Server cannot start without Elasticsearch connection");
        tracing::error!("Please ensure Elasticsearch is running on {}", elasticsearch_url);
        tracing::error!("Try: docker run -p 9200:9200 -e 'discovery.type=single-node' elasticsearch:8.15.0");
        return Err(anyhow::anyhow!("Elasticsearch connection failed - all connection methods failed"));
    }
    
    tracing::info!("✅ Elasticsearch connection verified successfully");

    // Shared application state
    let app_state = AppState {
        db: Arc::new(pool),
        elasticsearch: Arc::new(elasticsearch_client),
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
        .nest("/api", routes::knowledge::create_knowledge_router())
        .nest("/api", routes::query::create_query_router())
        .nest("/api", routes::chat::create_chat_router())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .with_state(app_state);

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("🌍 Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}