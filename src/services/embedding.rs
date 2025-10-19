use anyhow::Result;
use embed_anything::{
    embed_file,
    embeddings::embed::{Embedder, TextEmbedder},
    embeddings::local::bert::BertEmbedder,
};
use qdrant_client::{
    qdrant::{
        vectors_config::Config,
        CreateCollection, Distance, PointStruct, VectorParams, VectorsConfig,
        SearchPoints,
    },
    Qdrant,
};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing;
use uuid::Uuid;

pub struct EmbeddingService {
    qdrant: Qdrant,
}

impl EmbeddingService {
    pub fn new(qdrant: &Qdrant) -> Result<Self> {
        Ok(Self {
            qdrant: qdrant.clone(),
        })
    }

    // Create a collection for a chatbot if it doesn't exist
    pub async fn create_collection_if_not_exists(&self, collection_name: &str) -> Result<()> {
        tracing::info!("Checking if collection '{}' exists", collection_name);

        // Check if collection exists
        match self.qdrant.list_collections().await {
            Ok(collections) => {
                let exists = collections
                    .collections
                    .iter()
                    .any(|c| c.name == collection_name);

                if exists {
                    tracing::info!("Collection '{}' already exists", collection_name);
                    return Ok(());
                }
            }
            Err(e) => {
                tracing::warn!("Failed to list collections: {}", e);
            }
        }

        // Create collection
        tracing::info!("Creating collection '{}'", collection_name);
        
        let collection_config = CreateCollection {
            collection_name: collection_name.to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: 384, // BERT embedding size
                    distance: Distance::Cosine as i32,
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        self.qdrant.create_collection(collection_config).await?;
        tracing::info!("✅ Collection '{}' created successfully", collection_name);

        Ok(())
    }

    // Process PDF file and create embeddings
    pub async fn process_pdf_file(
        &self,
        file_path: &PathBuf,
        collection_name: &str,
    ) -> Result<usize> {
        tracing::info!("Processing PDF file: {:?}", file_path);

        // For now, let's create a simple test implementation
        // We'll need to debug the actual embed_anything structure
        tracing::warn!("PDF processing temporarily disabled - need to debug embed_anything structure");
        
        // Create a dummy embedding for testing
        let dummy_embedding = vec![0.1; 384]; // BERT embedding size
        
        // Create a test point
        let point = PointStruct {
            id: Some(Uuid::new_v4().to_string().into()),
            vectors: Some(dummy_embedding.into()),
            payload: {
                let mut payload = HashMap::new();
                payload.insert("text".to_string(), "Test PDF content".to_string().into());
                payload.insert("chunk_index".to_string(), 0i64.into());
                payload.insert("file_path".to_string(), file_path.to_string_lossy().to_string().into());
                payload
            },
            ..Default::default()
        };

        // Insert the test point
        let upsert_points = qdrant_client::qdrant::UpsertPoints {
            collection_name: collection_name.to_string(),
            points: vec![point],
            ..Default::default()
        };

        self.qdrant.upsert_points(upsert_points).await?;
        
        tracing::info!("✅ Successfully stored test embedding in collection '{}'", collection_name);
        Ok(1)
    }

    // Search for similar embeddings
    pub async fn search_similar(
        &self,
        collection_name: &str,
        query_text: &str,
        limit: u64,
    ) -> Result<Vec<qdrant_client::qdrant::ScoredPoint>> {
        tracing::info!("Searching for similar embeddings in collection '{}'", collection_name);

        // For now, create a dummy query vector
        let query_vector = vec![0.1; 384]; // BERT embedding size

        // Search in Qdrant
        let search_points = SearchPoints {
            collection_name: collection_name.to_string(),
            vector: query_vector,
            limit,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        let search_result = self.qdrant.search_points(search_points).await?;
        
        tracing::info!("Found {} similar points", search_result.result.len());

        Ok(search_result.result)
    }
}
