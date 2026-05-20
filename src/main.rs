mod api;
mod dal;
mod domain;
mod services;
mod generated;
mod infrastructure;

// #[cfg(test)]
// mod tests;

use api::routers::build_router;
use infrastructure::grpc::client::AuthGrpcClient;
use http::HeaderValue;
use tower_http::cors::{CorsLayer, AllowOrigin};
use http::Method;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use http::header::{AUTHORIZATION, ACCEPT, CONTENT_TYPE};
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // ── File-based logging (daily rotation, no console output) ───────────────
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    let file_appender = tracing_appender::rolling::daily(&log_dir, "core-backend.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "core_backend=debug,info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(false),
        )
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %log_dir,
        "core-backend starting"
    );

    let postgres = PgPool::connect(
        &std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
    )
    .await
    .expect("Failed to connect to Postgres");

    tracing::info!("database pool ready");

    let security_grpc_addr = std::env::var("SECURITY_GRPC_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    let auth_client = if std::env::var("GRPC_TLS_ENABLED").unwrap_or_default() == "true" {
        let ca_path   = std::env::var("GRPC_CA_CERT").expect("GRPC_CA_CERT must be set when GRPC_TLS_ENABLED=true");
        let cert_path = std::env::var("GRPC_CLIENT_CERT").expect("GRPC_CLIENT_CERT must be set when GRPC_TLS_ENABLED=true");
        let key_path  = std::env::var("GRPC_CLIENT_KEY").expect("GRPC_CLIENT_KEY must be set when GRPC_TLS_ENABLED=true");
        let domain    = std::env::var("GRPC_DOMAIN").unwrap_or_else(|_| "security-core".to_string());

        let ca   = std::fs::read(&ca_path).expect("failed to read GRPC_CA_CERT file");
        let cert = std::fs::read(&cert_path).expect("failed to read GRPC_CLIENT_CERT file");
        let key  = std::fs::read(&key_path).expect("failed to read GRPC_CLIENT_KEY file");

        AuthGrpcClient::connect_mtls(&security_grpc_addr, &domain, &ca, &cert, &key)
            .await
            .expect("failed to connect to security-core gRPC (mTLS)")
    } else {
        AuthGrpcClient::connect_insecure(&security_grpc_addr)
            .await
            .expect("failed to connect to security-core gRPC (insecure)")
    };

    let allowed_origins = [
        "http://localhost:3001".parse::<HeaderValue>().unwrap(),
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "https://dashboard.soflib.com".parse::<HeaderValue>().unwrap(),
        "https://arch.pilot.soflib.com".parse::<HeaderValue>().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = build_router(postgres, auth_client).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2009")
        .await
        .unwrap();

    tracing::info!(addr = "0.0.0.0:2009", "HTTP server ready");
    tracing::info!(url = "http://localhost:2009/swagger-ui/", "Swagger UI available");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}