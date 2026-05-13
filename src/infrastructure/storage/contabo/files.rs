// S3-compatible file operations against the configured Contabo bucket.
//
// Operations:
//   ensure_bucket()                  — create the bucket if it does not exist (run once at startup)
//   upload(key, data, content_type)  — PUT an object
//   download(key)                    — GET an object → bytes
//   delete_object(key)               — DELETE an object
//   list_keys(prefix)                — LIST objects by prefix → Vec<key>
//   presigned_get(key, secs)         — temporary read URL
//   presigned_put(key, secs)         — temporary upload URL
//
// Build keys with the helpers in the `keys` submodule:
//   keys::foto_progreso(proyecto_id, filename)
//   keys::plano(proyecto_id, filename)
//   keys::factura_egreso(proyecto_id, egreso_id, filename)
//   keys::cotizacion(ppto_id, filename)
//   etc.

use s3::{BucketConfiguration, creds::Credentials, Region};
use super::ContaboStorage;

impl ContaboStorage {
    /// Create the configured bucket if it does not already exist.
    /// Safe to call on every startup; returns Ok if created (200) or already exists (409).
    pub async fn ensure_bucket(&self) -> Result<(), String> {
        let creds = Credentials::new(
            Some(&self.config.s3_access_key),
            Some(&self.config.s3_secret_key),
            None, None, None,
        ).map_err(|e| format!("ensure_bucket creds: {e}"))?;

        let region = Region::Custom {
            region:   self.config.s3_region.clone(),
            endpoint: self.config.s3_endpoint.clone(),
        };

        let resp = s3::Bucket::create_with_path_style(
            &self.config.s3_bucket,
            region,
            creds,
            BucketConfiguration::default(),
        ).await.map_err(|e| format!("ensure_bucket: {e}"))?;

        // 200 = created, 409 = already owned by you — both are fine
        if resp.response_code == 200 || resp.response_code == 409 {
            Ok(())
        } else {
            Err(format!("ensure_bucket returned HTTP {}", resp.response_code))
        }
    }

    /// Upload raw bytes to the given object key.
    /// Build the key with helpers from the `keys` module, e.g.:
    ///   `keys::foto_progreso(proyecto_id, &filename)`
    pub async fn upload(
        &self,
        key:          &str,
        data:         &[u8],
        content_type: &str,
    ) -> Result<(), String> {
        self.bucket
            .put_object_with_content_type(key, data, content_type)
            .await
            .map_err(|e| format!("upload '{key}': {e}"))?;
        Ok(())
    }

    /// Download an object and return its bytes.
    pub async fn download(&self, key: &str) -> Result<Vec<u8>, String> {
        let resp = self.bucket
            .get_object(key)
            .await
            .map_err(|e| format!("download '{key}': {e}"))?;
        Ok(resp.bytes().to_vec())
    }

    /// Delete a single object.
    pub async fn delete_object(&self, key: &str) -> Result<(), String> {
        self.bucket
            .delete_object(key)
            .await
            .map_err(|e| format!("delete '{key}': {e}"))?;
        Ok(())
    }

    /// List all object keys that start with `prefix`.
    /// Use `keys::proyecto_prefix(id)` or `keys::fotos_prefix(id)` to scope the listing.
    pub async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, String> {
        let pages = self.bucket
            .list(prefix.to_string(), None)
            .await
            .map_err(|e| format!("list '{prefix}': {e}"))?;
        Ok(pages
            .into_iter()
            .flat_map(|page| page.contents)
            .map(|obj| obj.key)
            .collect())
    }

    /// Generate a pre-signed GET URL valid for `expires_sec` seconds.
    /// Use this to give the frontend a temporary read link without exposing credentials.
    pub async fn presigned_get(&self, key: &str, expires_sec: u32) -> Result<String, String> {
        self.bucket
            .presign_get(key, expires_sec, None)
            .await
            .map_err(|e| format!("presign_get '{key}': {e}"))
    }

    /// Generate a pre-signed PUT URL valid for `expires_sec` seconds.
    /// Use this to let clients upload directly to Contabo without routing bytes through the API.
    pub async fn presigned_put(&self, key: &str, expires_sec: u32) -> Result<String, String> {
        self.bucket
            .presign_put(key, expires_sec, None, None)
            .await
            .map_err(|e| format!("presign_put '{key}': {e}"))
    }
}
