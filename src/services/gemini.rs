use crate::errors::AppResult;
use gemini_rust::Gemini;
use serde::Serialize;
use std::env;
use futures_util::{Stream, stream, TryStreamExt};
use std::pin::Pin;

#[derive(Debug, Serialize)]
pub struct StreamingChunk {
    pub text: String,
    pub is_final: bool,
}

pub struct GeminiService {
    client: Gemini,
}

impl GeminiService {
    pub fn new() -> AppResult<Self> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| crate::errors::AppError::Other("GEMINI_API_KEY environment variable not set".to_string()))?;
        
        let client = Gemini::new(api_key)
            .map_err(|e| crate::errors::AppError::Other(format!("Failed to create Gemini client: {}", e)))?;
        
        Ok(Self {
            client,
        })
    }

    pub async fn generate_response(&self, user_query: &str, context: &str) -> AppResult<String> {
        let prompt = format!(
            "You are a helpful AI assistant. Based on the following context, please answer the user's question. If the context doesn't contain enough information to answer the question, please say so.\n\nContext:\n{}\n\nUser Question: {}\n\nAnswer:",
            context,
            user_query
        );

        tracing::info!("Sending request to Gemini API");

        let response = self.client
            .generate_content()
            .with_user_message(&prompt)
            .execute()
            .await
            .map_err(|e| crate::errors::AppError::Other(format!("Gemini API error: {}", e)))?;

        let response_text = response.text();
        tracing::info!("✅ Generated response from Gemini API");

        Ok(response_text)
    }

    pub async fn generate_response_stream(
        &self,
        user_query: &str,
        context: &str,
    ) -> AppResult<Pin<Box<dyn Stream<Item = AppResult<StreamingChunk>> + Send>>> {
        let prompt = format!(
            "You are a helpful AI assistant. Based on the following context, please answer the user's question. If the context doesn't contain enough information to answer the question, please say so.\n\nContext:\n{}\n\nUser Question: {}\n\nAnswer:",
            context,
            user_query
        );

        tracing::info!("Starting streaming request to Gemini API");

        let gemini_stream = self.client
            .generate_content()
            .with_user_message(&prompt)
            .execute_stream()
            .await
            .map_err(|e| crate::errors::AppError::Other(format!("Failed to start Gemini streaming: {}", e)))?;

        let stream = stream::unfold(gemini_stream, |mut stream| async move {
            match stream.try_next().await {
                Ok(Some(chunk)) => {
                    let chunk_text = chunk.text();
                    // For now, we'll assume each chunk is not final unless it's empty
                    // This might need adjustment based on the actual API
                    let is_final = chunk_text.is_empty();
                    
                    Some((
                        Ok(StreamingChunk {
                            text: chunk_text,
                            is_final,
                        }),
                        stream,
                    ))
                }
                Ok(None) => {
                    // Stream ended - send final chunk
                    Some((
                        Ok(StreamingChunk {
                            text: "".to_string(),
                            is_final: true,
                        }),
                        stream,
                    ))
                }
                Err(e) => {
                    tracing::error!("Streaming error: {}", e);
                    Some((
                        Err(crate::errors::AppError::Other(format!("Streaming error: {}", e))),
                        stream,
                    ))
                }
            }
        });

        Ok(Box::pin(stream))
    }
}
