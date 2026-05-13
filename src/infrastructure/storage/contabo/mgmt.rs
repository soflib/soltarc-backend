// Contabo management API — https://api.contabo.com/v1/object-storages
//
// Endpoints covered:
//   GET  /object-storages                          list_storages()
//   GET  /object-storages/{id}                     get_storage()
//   GET  /object-storages/{id}/stats               get_stats()
//   GET  /object-storages/data-centers             list_data_centers()
//
// Every request sends a fresh x-request-id UUID as required by the API.

use serde::Deserialize;
use super::{ContaboStorage, MGMT_BASE};

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStorageInfo {
    pub object_storage_id:         String,
    pub status:                    String,
    #[serde(default)]
    pub display_name:              Option<String>,
    #[serde(default)]
    pub s3_url:                    Option<String>,
    #[serde(default)]
    pub total_purchased_space_tb:  Option<f64>,
    #[serde(default)]
    pub data_center:               Option<String>,
    #[serde(default)]
    pub region:                    Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageStats {
    #[serde(default)]
    pub total_space_gb:  Option<f64>,
    #[serde(default)]
    pub used_space_gb:   Option<f64>,
    #[serde(default)]
    pub num_of_files:    Option<i64>,
    #[serde(default)]
    pub num_of_buckets:  Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCenter {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub s3_url: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
}

#[derive(Deserialize)]
struct List<T> { data: Vec<T> }

// ── Helpers ───────────────────────────────────────────────────────────────────

fn req_id() -> String { uuid::Uuid::new_v4().to_string() }

async fn check(res: reqwest::Response, ctx: &str) -> Result<reqwest::Response, String> {
    if res.status().is_success() {
        Ok(res)
    } else {
        let s = res.status();
        let b = res.text().await.unwrap_or_default();
        Err(format!("{ctx} {s}: {b}"))
    }
}

// ── Management operations ─────────────────────────────────────────────────────

impl ContaboStorage {
    /// List all object storage instances in the account.
    pub async fn list_storages(&self) -> Result<Vec<ObjectStorageInfo>, String> {
        let tok = self.bearer_token().await?;
        let res = self.http
            .get(format!("{MGMT_BASE}/object-storages"))
            .bearer_auth(&tok)
            .header("x-request-id", req_id())
            .send().await
            .map_err(|e| e.to_string())?;
        Ok(check(res, "list_storages").await?.json::<List<ObjectStorageInfo>>()
            .await.map_err(|e| e.to_string())?.data)
    }

    /// Get details for the configured object storage instance.
    pub async fn get_storage(&self) -> Result<ObjectStorageInfo, String> {
        let tok = self.bearer_token().await?;
        let id  = &self.config.object_storage_id;
        let res = self.http
            .get(format!("{MGMT_BASE}/object-storages/{id}"))
            .bearer_auth(&tok)
            .header("x-request-id", req_id())
            .send().await
            .map_err(|e| e.to_string())?;
        check(res, "get_storage").await?
            .json::<List<ObjectStorageInfo>>().await.map_err(|e| e.to_string())?
            .data.into_iter().next()
            .ok_or_else(|| "object storage not found".to_string())
    }

    /// Get usage statistics for the configured object storage instance.
    pub async fn get_stats(&self) -> Result<StorageStats, String> {
        let tok = self.bearer_token().await?;
        let id  = &self.config.object_storage_id;
        let res = self.http
            .get(format!("{MGMT_BASE}/object-storages/{id}/stats"))
            .bearer_auth(&tok)
            .header("x-request-id", req_id())
            .send().await
            .map_err(|e| e.to_string())?;
        check(res, "get_stats").await?
            .json::<List<StorageStats>>().await.map_err(|e| e.to_string())?
            .data.into_iter().next()
            .ok_or_else(|| "no stats returned".to_string())
    }

    /// List all available Contabo data centers with their S3 endpoint URLs.
    pub async fn list_data_centers(&self) -> Result<Vec<DataCenter>, String> {
        let tok = self.bearer_token().await?;
        let res = self.http
            .get(format!("{MGMT_BASE}/object-storages/data-centers"))
            .bearer_auth(&tok)
            .header("x-request-id", req_id())
            .send().await
            .map_err(|e| e.to_string())?;
        Ok(check(res, "list_data_centers").await?
            .json::<List<DataCenter>>().await.map_err(|e| e.to_string())?.data)
    }
}
