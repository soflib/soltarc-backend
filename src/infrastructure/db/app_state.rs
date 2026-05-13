use sqlx::PgPool;
use crate::infrastructure::grpc::client::AuthGrpcClient;

#[derive(Clone)]
pub struct AppState {
    pub postgres:    PgPool,
    pub auth_grpc:   AuthGrpcClient,
}