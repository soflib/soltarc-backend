// Contabo Object Storage client.
//
// Two planes:
//   - Management API  (https://api.contabo.com/v1) — list storages, stats,
//     data centers.  Uses OAuth2 password-grant; tokens are cached.
//   - S3-compatible API — upload / download / delete / list / presign.
//     Uses rust-s3 with a custom endpoint + path-style addressing.
//
// Required env vars (see .env):
//   CONTABO_CLIENT_ID, CONTABO_CLIENT_SECRET, CONTABO_USERNAME,
//   CONTABO_PASSWORD, CONTABO_OBJECT_STORAGE_ID,
//   CONTABO_S3_ENDPOINT, CONTABO_S3_ACCESS_KEY, CONTABO_S3_SECRET_KEY,
//   CONTABO_S3_BUCKET, CONTABO_S3_REGION (optional, default "eu2")

mod auth;
pub mod files;
pub mod keys;
pub mod mgmt;

use reqwest::Client;
use s3::{creds::Credentials, Bucket, Region};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) const MGMT_BASE: &str = "https://api.contabo.com/v1";

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ContaboConfig {
    pub client_id:         String,
    pub client_secret:     String,
    pub username:          String,
    pub password:          String,
    pub object_storage_id: String,
    pub s3_endpoint:       String,
    pub s3_access_key:     String,
    pub s3_secret_key:     String,
    pub s3_bucket:         String,
    pub s3_region:         String,
}

impl ContaboConfig {
    pub fn from_env() -> Result<Self, String> {
        fn v(k: &str) -> Result<String, String> {
            std::env::var(k).map_err(|_| format!("missing env var: {k}"))
        }
        Ok(Self {
            client_id:         v("CONTABO_CLIENT_ID")?,
            client_secret:     v("CONTABO_CLIENT_SECRET")?,
            username:          v("CONTABO_USERNAME")?,
            password:          v("CONTABO_PASSWORD")?,
            object_storage_id: v("CONTABO_OBJECT_STORAGE_ID")?,
            s3_endpoint:       v("CONTABO_S3_ENDPOINT")?,
            s3_access_key:     v("CONTABO_S3_ACCESS_KEY")?,
            s3_secret_key:     v("CONTABO_S3_SECRET_KEY")?,
            s3_bucket:         v("CONTABO_S3_BUCKET")?,
            s3_region:         std::env::var("CONTABO_S3_REGION")
                                   .unwrap_or_else(|_| "eu2".to_string()),
        })
    }
}

// ── Client ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ContaboStorage {
    pub(crate) http:   Client,
    pub(crate) bucket: Arc<Bucket>,
    pub(crate) config: ContaboConfig,
    pub(crate) token:  Arc<Mutex<auth::TokenState>>,
}

impl ContaboStorage {
    pub fn new(config: ContaboConfig) -> Result<Self, String> {
        let creds = Credentials::new(
            Some(&config.s3_access_key),
            Some(&config.s3_secret_key),
            None, None, None,
        ).map_err(|e| format!("s3 credentials: {e}"))?;

        let region = Region::Custom {
            region:   config.s3_region.clone(),
            endpoint: config.s3_endpoint.clone(),
        };

        // with_path_style() is required for Contabo S3-compatible endpoints
        let bucket = Bucket::new(&config.s3_bucket, region, creds)
            .map_err(|e| format!("s3 bucket init: {e}"))?
            .with_path_style();

        Ok(Self {
            http:   Client::new(),
            bucket: Arc::from(bucket),
            config,
            token:  Arc::new(Mutex::new(auth::TokenState::default())),
        })
    }

    pub fn from_env() -> Result<Self, String> {
        Self::new(ContaboConfig::from_env()?)
    }

    /// Returns a valid management-API bearer token, refreshing if needed.
    pub(crate) async fn bearer_token(&self) -> Result<String, String> {
        let mut s = self.token.lock().await;
        if s.is_valid() {
            return Ok(s.get().to_owned());
        }
        let c = &self.config;
        let (tok, exp) = auth::fetch(
            &self.http,
            &c.client_id,
            &c.client_secret,
            &c.username,
            &c.password,
        ).await?;
        s.set(tok.clone(), exp);
        Ok(tok)
    }
}
