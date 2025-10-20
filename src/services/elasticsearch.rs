use anyhow::Result;
use elasticsearch::{
    indices::{IndicesCreateParts, IndicesExistsParts},
    Elasticsearch, SearchParts,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing;

pub struct ElasticsearchService {
    client: Arc<Elasticsearch>,
}

impl ElasticsearchService {
    pub fn new(client: Arc<Elasticsearch>) -> Self {
        Self { client }
    }

    // Create an index for a chatbot if it doesn't exist
    pub async fn create_index_if_not_exists(&self, index_name: &str, embedding_dim: usize) -> Result<()> {
        tracing::info!("Checking if index '{}' exists", index_name);

        // Check if index exists
        let exists_response = self
            .client
            .indices()
            .exists(IndicesExistsParts::Index(&[index_name]))
            .send()
            .await?;

        if exists_response.status_code().is_success() {
            tracing::info!("Index '{}' already exists", index_name);
            return Ok(());
        }

        // Create index with mapping for dense vector
        let mapping = json!({
            "mappings": {
                "properties": {
                    "text": {
                        "type": "text",
                        "analyzer": "standard"
                    },
                    "embedding": {
                        "type": "dense_vector",
                        "dims": embedding_dim,
                        "index": true,
                        "similarity": "cosine"
                    },
                    "chunk_index": {
                        "type": "long"
                    },
                    "file_path": {
                        "type": "keyword"
                    },
                    "chunk_count": {
                        "type": "long"
                    },
                    "created_at": {
                        "type": "date"
                    }
                }
            },
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0
            }
        });

        let create_response = self
            .client
            .indices()
            .create(IndicesCreateParts::Index(index_name))
            .body(mapping)
            .send()
            .await?;

        if create_response.status_code().is_success() {
            tracing::info!("✅ Index '{}' created successfully", index_name);
        } else {
            let error_text = create_response.text().await?;
            tracing::error!("Failed to create index '{}': {}", index_name, error_text);
            return Err(anyhow::anyhow!("Failed to create index"));
        }

        Ok(())
    }

    // Index documents with embeddings
    pub async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<DocumentWithEmbedding>,
    ) -> Result<usize> {
        let total_docs = documents.len();
        tracing::info!("Indexing {} documents to index '{}'", total_docs, index_name);

        let mut success_count = 0;

        for doc in documents {
            let document_body = json!({
                "text": doc.text,
                "embedding": doc.embedding,
                "chunk_index": doc.chunk_index,
                "file_path": doc.file_path,
                "chunk_count": doc.chunk_count,
                "created_at": chrono::Utc::now().to_rfc3339()
            });

            let response = self
                .client
                .index(elasticsearch::IndexParts::IndexId(index_name, &doc.id))
                .body(document_body)
                .send()
                .await?;

            if response.status_code().is_success() {
                success_count += 1;
            } else {
                let error_text = response.text().await?;
                tracing::warn!("Failed to index document {}: {}", doc.id, error_text);
            }
        }

        tracing::info!("✅ Successfully indexed {} out of {} documents", success_count, total_docs);
        Ok(success_count)
    }

    // Search for similar documents using vector similarity
    pub async fn search_similar(
        &self,
        index_name: &str,
        query_embedding: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<SearchResult>> {
        tracing::info!("Searching for similar documents in index '{}'", index_name);

        let search_query = json!({
            "knn": {
                "field": "embedding",
                "query_vector": query_embedding,
                "k": limit,
                "num_candidates": limit * 2
            },
            "_source": ["text", "chunk_index", "file_path", "chunk_count"]
        });

        let response = self
            .client
            .search(SearchParts::Index(&[index_name]))
            .body(search_query)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await?;
            tracing::error!("Search failed: {}", error_text);
            return Err(anyhow::anyhow!("Search failed"));
        }

        let response_body: Value = response.json().await?;
        let empty_vec = vec![];
        let hits = response_body["hits"]["hits"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for hit in hits {
            let source = &hit["_source"];
            let score = hit["_score"].as_f64().unwrap_or(0.0) as f32;

            results.push(SearchResult {
                text: source["text"].as_str().unwrap_or("").to_string(),
                score,
                chunk_index: source["chunk_index"].as_i64().unwrap_or(0),
                file_path: source["file_path"].as_str().unwrap_or("").to_string(),
            });
        }

        tracing::info!("Found {} similar documents", results.len());
        Ok(results)
    }
}

#[derive(Debug)]
pub struct DocumentWithEmbedding {
    pub id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub chunk_index: i64,
    pub file_path: String,
    pub chunk_count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct SearchResult {
    pub text: String,
    pub score: f32,
    pub chunk_index: i64,
    pub file_path: String,
}
