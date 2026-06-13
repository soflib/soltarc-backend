use sqlx::PgPool;
use std::sync::Arc;
use crate::infrastructure::grpc::client::AuthGrpcClient;
use crate::infrastructure::storage::contabo::ContaboStorage;

#[derive(Clone)]
pub struct AppState {
    pub postgres:    PgPool,
    pub auth_grpc:   AuthGrpcClient,
    /// Object storage (Contabo). None si faltan los env CONTABO_S3_* —
    /// los endpoints de archivos responden 503 en ese caso (no panic).
    pub storage:     Option<Arc<ContaboStorage>>,
}