use anyhow::Result;
use elasticsearch::Elasticsearch;
use std::path::PathBuf;
use std::sync::Arc;
use tracing;
use uuid::Uuid;

use crate::services::candle_embedding::{CandleEmbeddingService, EmbeddingConfig};
use crate::services::elasticsearch::{DocumentWithEmbedding, ElasticsearchService};
use crate::utils::pdf::process_pdf_file;

pub struct EmbeddingService {
    elasticsearch_service: ElasticsearchService,
    candle_service: CandleEmbeddingService,
}

impl EmbeddingService {
    pub fn new(elasticsearch: Arc<Elasticsearch>) -> Result<Self> {
        tracing::info!("Initializing EmbeddingService with Elasticsearch backend");
        
        // Initialize Candle embedding service
        let config = EmbeddingConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_length: 512,
            embedding_dim: 384,
        };
        
        let candle_service = CandleEmbeddingService::new(Some(config))?;
        let elasticsearch_service = ElasticsearchService::new(elasticsearch);
        
        Ok(Self {
            elasticsearch_service,
            candle_service,
        })
    }

    // Create an index for a chatbot if it doesn't exist
    pub async fn create_collection_if_not_exists(&self, collection_name: &str) -> Result<()> {
        self.elasticsearch_service
            .create_index_if_not_exists(collection_name, self.candle_service.embedding_dim())
            .await
    }

    // Process PDF file and create embeddings
    pub async fn process_pdf_file(
        &self,
        file_path: &PathBuf,
        collection_name: &str,
    ) -> Result<usize> {
        tracing::info!("Processing PDF file: {:?}", file_path);

        // Extract text from PDF and chunk it
        let chunks = process_pdf_file(file_path, 200, 50)?; // 200 words per chunk, 50 word overlap
        
        if chunks.is_empty() {
            tracing::warn!("No text chunks extracted from PDF");
            return Ok(0);
        }

        tracing::info!("Extracted {} text chunks from PDF", chunks.len());

        // Generate embeddings for all chunks
        let embeddings = self.candle_service.embed_texts(&chunks)?;
        
        if embeddings.len() != chunks.len() {
            tracing::error!("Mismatch between chunks ({}) and embeddings ({})", chunks.len(), embeddings.len());
            return Err(anyhow::anyhow!("Embedding generation failed"));
        }

        // Create documents for Elasticsearch
        let mut documents = Vec::new();
        
        for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            let document = DocumentWithEmbedding {
                id: Uuid::new_v4().to_string(),
                text: chunk.clone(),
                embedding: embedding.clone(),
                chunk_index: i as i64,
                file_path: file_path.to_string_lossy().to_string(),
                chunk_count: chunks.len() as i64,
            };
            documents.push(document);
        }

        // Index all documents in Elasticsearch
        let indexed_count = self.elasticsearch_service
            .index_documents(collection_name, documents)
            .await?;
        
        tracing::info!("✅ Successfully stored {} embeddings in index '{}'", indexed_count, collection_name);
        
        Ok(indexed_count)
    }

    // Search for similar embeddings
    pub async fn search_similar(
        &self,
        collection_name: &str,
        query_text: &str,
        limit: u64,
    ) -> Result<Vec<crate::services::elasticsearch::SearchResult>> {
        tracing::info!("Searching for similar embeddings in index '{}'", collection_name);

        // Generate embedding for the query text
        let query_embedding = self.candle_service.embed_text(query_text)?;

        // Search in Elasticsearch
        let search_results = self.elasticsearch_service
            .search_similar(collection_name, query_embedding, limit)
            .await?;
        
        tracing::info!("Found {} similar documents", search_results.len());

        Ok(search_results)
    }

    // Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.candle_service.embedding_dim()
    }
}
