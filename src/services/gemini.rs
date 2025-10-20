use crate::errors::AppResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<RequestContent>,
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct RequestContent {
    parts: Vec<RequestPart>,
}

#[derive(Debug, Serialize)]
struct RequestPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    top_k: i32,
    top_p: f32,
    max_output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
    #[serde(rename = "finishReason")]
    finish_reason: String,
    index: i32,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
    role: String,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: String,
}

pub struct GeminiService {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiService {
    pub fn new() -> AppResult<Self> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| crate::errors::AppError::Other("GEMINI_API_KEY environment variable not set".to_string()))?;
        
        let model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
        
        Ok(Self {
            client: Client::new(),
            api_key,
            model,
        })
    }

    pub async fn generate_response(&self, user_query: &str, context: &str) -> AppResult<String> {
        let prompt = format!(
            "You are a helpful AI assistant. Based on the following context, please answer the user's question. If the context doesn't contain enough information to answer the question, please say so.\n\nContext:\n{}\n\nUser Question: {}\n\nAnswer:",
            context,
            user_query
        );

        let request_body = GeminiRequest {
            contents: vec![RequestContent {
                parts: vec![RequestPart { text: prompt }],
            }],
            generation_config: GenerationConfig {
                temperature: 0.7,
                top_k: 40,
                top_p: 0.95,
                max_output_tokens: 1024,
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        tracing::info!("Sending request to Gemini API");

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = match response.text().await {
                Ok(text) => text,
                Err(_) => "Unknown error".to_string(),
            };
            return Err(crate::errors::AppError::Other(format!(
                "Gemini API returned error status {}: {}",
                status,
                error_text
            )));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await?;

        if gemini_response.candidates.is_empty() {
            return Err(crate::errors::AppError::Other("No candidates returned from Gemini API".to_string()));
        }

        let candidate = &gemini_response.candidates[0];
        if candidate.content.parts.is_empty() {
            return Err(crate::errors::AppError::Other("No content parts returned from Gemini API".to_string()));
        }

        let response_text = candidate.content.parts[0].text.clone();
        tracing::info!("✅ Generated response from Gemini API");

        Ok(response_text)
    }
}
