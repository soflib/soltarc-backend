// Contabo Object Storage client.
//
// Two planes:
//   - Management API  (https://api.contabo.com/v1) — list storages, stats,
//     data centers.  Uses OAuth2 password-grant; tokens are cached.
//   - S3-compatible API — upload / download / delete / list / presign.
//     Uses rust-s3 with a custom endpoint + path-style addressing.
//
// Env vars (see .env):
//   Required (S3 file I/O):
//     CONTABO_S3_ENDPOINT, CONTABO_S3_ACCESS_KEY, CONTABO_S3_SECRET_KEY,
//     CONTABO_S3_BUCKET
//     CONTABO_S3_REGION (optional, default "eu2")
//   Optional (management API only — list storages / stats / data centers;
//   not needed for file I/O):
//     CONTABO_CLIENT_ID, CONTABO_CLIENT_SECRET, CONTABO_USERNAME,
//     CONTABO_PASSWORD, CONTABO_OBJECT_STORAGE_ID

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
    // Management-API credentials — optional; only needed for the (currently
    // unrouted) mgmt endpoints. None when the CONTABO_CLIENT_* vars are unset.
    pub client_id:         Option<String>,
    pub client_secret:     Option<String>,
    pub username:          Option<String>,
    pub password:          Option<String>,
    pub object_storage_id: Option<String>,
    // S3 credentials — required for file I/O.
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
            client_id:         std::env::var("CONTABO_CLIENT_ID").ok(),
            client_secret:     std::env::var("CONTABO_CLIENT_SECRET").ok(),
            username:          std::env::var("CONTABO_USERNAME").ok(),
            password:          std::env::var("CONTABO_PASSWORD").ok(),
            object_storage_id: std::env::var("CONTABO_OBJECT_STORAGE_ID").ok(),
            s3_endpoint:       v("CONTABO_S3_ENDPOINT")?,
            s3_access_key:     v("CONTABO_S3_ACCESS_KEY")?,
            s3_secret_key:     v("CONTABO_S3_SECRET_KEY")?,
            s3_bucket:         v("CONTABO_S3_BUCKET")?,
            s3_region:         std::env::var("CONTABO_S3_REGION")
                                   .unwrap_or_else(|_| "eu2".to_string()),
        })
    }

    /// The five management-API credentials, validated as all-present.
    /// Returns `(client_id, client_secret, username, password, object_storage_id)`
    /// or an error if the management plane is not configured.
    pub(crate) fn mgmt_creds(&self) -> Result<(&str, &str, &str, &str, &str), String> {
        match (&self.client_id, &self.client_secret, &self.username,
               &self.password, &self.object_storage_id) {
            (Some(a), Some(b), Some(c), Some(d), Some(e)) =>
                Ok((a, b, c, d, e)),
            _ => Err("contabo management API not configured (set \
                      CONTABO_CLIENT_ID/CLIENT_SECRET/USERNAME/PASSWORD/\
                      OBJECT_STORAGE_ID)".into()),
        }
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
        let (cid, csecret, user, pass, _id) = self.config.mgmt_creds()?;
        let (tok, exp) = auth::fetch(
            &self.http,
            cid,
            csecret,
            user,
            pass,
        ).await?;
        s.set(tok.clone(), exp);
        Ok(tok)
    }
}
