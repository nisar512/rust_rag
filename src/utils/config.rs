use elasticsearch::Elasticsearch;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub elasticsearch: Arc<Elasticsearch>,
}
