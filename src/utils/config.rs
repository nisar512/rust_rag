use qdrant_client::Qdrant;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub qdrant: Arc<Qdrant>,
}
