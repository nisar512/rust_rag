use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::db::queries::get_chat_bot;
use crate::services::embedding::EmbeddingService;
use crate::utils::config::AppState;

// Upload PDF file and create embeddings for a chatbot
pub async fn upload_pdf_handler(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Starting PDF upload process");

    let mut chatbot_id: Option<Uuid> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        match field.name() {
            Some("chatbot_id") => {
                let chatbot_id_str = field.text().await.map_err(|e| {
                    tracing::error!("Failed to read chatbot_id: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                
                chatbot_id = Some(Uuid::parse_str(&chatbot_id_str).map_err(|e| {
                    tracing::error!("Invalid chatbot_id format: {}", e);
                    StatusCode::BAD_REQUEST
                })?);
            }
            Some("file") => {
                file_name = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    tracing::error!("Failed to read file data: {}", e);
                    StatusCode::BAD_REQUEST
                })?.to_vec());
            }
            _ => {
                tracing::warn!("Unknown field: {:?}", field.name());
            }
        }
    }

    // Validate required fields
    let chatbot_id = chatbot_id.ok_or_else(|| {
        tracing::error!("Missing chatbot_id in request");
        StatusCode::BAD_REQUEST
    })?;

    let file_data = file_data.ok_or_else(|| {
        tracing::error!("Missing file in request");
        StatusCode::BAD_REQUEST
    })?;

    let file_name = file_name.unwrap_or_else(|| "unknown.pdf".to_string());

    tracing::info!("Processing PDF for chatbot: {}, file: {}", chatbot_id, file_name);

    // Verify chatbot exists
    tracing::info!("Checking if chatbot exists in database...");
    match get_chat_bot(&app_state.db, chatbot_id).await {
        Ok(Some(chatbot)) => {
            tracing::info!("✅ Found chatbot: {}", chatbot.name);
        }
        Ok(None) => {
            tracing::error!("❌ Chatbot not found: {}", chatbot_id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("❌ Database error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Create temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("{}_{}", chatbot_id, file_name));
    
    // Write file to temp location
    fs::write(&temp_file_path, &file_data).await.map_err(|e| {
        tracing::error!("Failed to write temp file: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("File saved to temp location: {:?}", temp_file_path);

    // Process PDF and create embeddings using Candle
    let embedding_count = match process_pdf_and_create_embeddings(&app_state, chatbot_id, &temp_file_path, &file_name).await {
        Ok(count) => {
            tracing::info!("✅ Successfully processed PDF and created {} embeddings", count);
            count
        }
        Err(e) => {
            tracing::error!("❌ Failed to process PDF: {}", e);
            // Clean up temp file
            let _ = fs::remove_file(&temp_file_path).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Clean up temp file
    let _ = fs::remove_file(&temp_file_path).await;
    
    tracing::info!("✅ PDF upload and processing completed successfully");
    
    Ok(Json(json!({
        "success": true,
        "message": "PDF uploaded and processed successfully",
        "data": {
            "chatbot_id": chatbot_id,
            "file_name": file_name,
            "embedding_count": embedding_count,
            "note": "PDF processed using Candle ML framework"
        }
    })))
}

// Process PDF file and create embeddings in Qdrant
async fn process_pdf_and_create_embeddings(
    app_state: &AppState,
    chatbot_id: Uuid,
    file_path: &PathBuf,
    _file_name: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting PDF processing for chatbot: {}", chatbot_id);

    // Create embedding service
    let embedding_service = EmbeddingService::new(app_state.elasticsearch.clone())?;
    
    // Create collection name for this chatbot
    let collection_name = format!("chatbot_{}", chatbot_id);
    
    // Ensure collection exists
    embedding_service.create_collection_if_not_exists(&collection_name).await?;
    
    // Process PDF and create embeddings
    let embedding_count = embedding_service.process_pdf_file(file_path, &collection_name).await?;
    
    tracing::info!("Created {} embeddings for chatbot {} in collection {}", 
                   embedding_count, chatbot_id, collection_name);
    
    Ok(embedding_count)
}

// Test endpoint to debug multipart
pub async fn test_upload_handler(
    State(_app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Testing multipart upload");
    
    let mut fields_received = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let field_name = field.name().unwrap_or("unknown").to_string();
        tracing::info!("Received field: {}", field_name);
        
        if field_name == "chatbot_id" {
            let text = field.text().await.map_err(|e| {
                tracing::error!("Failed to read chatbot_id text: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            fields_received.push(format!("chatbot_id: {}", text));
        } else if field_name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let bytes = field.bytes().await.map_err(|e| {
                tracing::error!("Failed to read file bytes: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            fields_received.push(format!("file: {} ({} bytes)", file_name, bytes.len()));
        }
    }
    
    Ok(Json(json!({
        "success": true,
        "message": "Test upload successful",
        "fields_received": fields_received
    })))
}

// Simple upload handler for testing
pub async fn simple_upload_handler(
    State(_app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Simple upload handler called");
    
    let mut chatbot_id: Option<Uuid> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        match field.name() {
            Some("chatbot_id") => {
                let chatbot_id_str = field.text().await.map_err(|e| {
                    tracing::error!("Failed to read chatbot_id: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                
                chatbot_id = Some(Uuid::parse_str(&chatbot_id_str).map_err(|e| {
                    tracing::error!("Invalid chatbot_id format: {}", e);
                    StatusCode::BAD_REQUEST
                })?);
            }
            Some("file") => {
                file_name = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    tracing::error!("Failed to read file data: {}", e);
                    StatusCode::BAD_REQUEST
                })?.to_vec());
            }
            _ => {
                tracing::warn!("Unknown field: {:?}", field.name());
            }
        }
    }

    // Validate required fields
    let chatbot_id = chatbot_id.ok_or_else(|| {
        tracing::error!("Missing chatbot_id in request");
        StatusCode::BAD_REQUEST
    })?;

    let file_data = file_data.ok_or_else(|| {
        tracing::error!("Missing file in request");
        StatusCode::BAD_REQUEST
    })?;

    let file_name = file_name.unwrap_or_else(|| "unknown.pdf".to_string());

    tracing::info!("✅ Simple upload successful - chatbot: {}, file: {} ({} bytes)", 
                   chatbot_id, file_name, file_data.len());

    Ok(Json(json!({
        "success": true,
        "message": "Simple upload successful",
        "data": {
            "chatbot_id": chatbot_id,
            "file_name": file_name,
            "file_size": file_data.len()
        }
    })))
}

// Create the router for knowledge management routes
pub fn create_knowledge_router() -> Router<AppState> {
    Router::new()
        .route("/upload-pdf", post(upload_pdf_handler))
        .route("/test-upload", post(test_upload_handler))
        .route("/simple-upload", post(simple_upload_handler))
}
