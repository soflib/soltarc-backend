// Envío de correo vía Microsoft Graph (sendMail). Puerto de
// payments_backend/src/infrastructure/azure/outlook.py.
//
// Flujo (mismo patrón token→API que infrastructure/storage/contabo/auth.rs):
//   1) OAuth client_credentials → access_token
//   2) POST /users/{sender}/sendMail con Bearer
//
// Reusa la MISMA app de Azure que payments_backend (necesita el permiso de
// aplicación Mail.Send). Si falta config, devuelve Err y el llamador degrada.

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use super::templates;

struct OutlookCfg {
    tenant:        String,
    client_id:     String,
    client_secret: String,
    sender:        String,
}

impl OutlookCfg {
    fn from_env() -> Result<Self, String> {
        fn v(k: &str) -> Result<String, String> {
            std::env::var(k)
                .ok()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| format!("falta env var de Outlook: {k}"))
        }
        Ok(Self {
            tenant:        v("DIRECTORY_TENANT_ID")?,
            client_id:     v("APPLICATION_CLIENT_ID")?,
            client_secret: v("VALUE")?,
            sender:        v("OUTLOOK_EMAIL_ADDRESS")?,
        })
    }
}

#[derive(Deserialize)]
struct TokenResp {
    access_token: String,
}

/// Percent-encode para body application/x-www-form-urlencoded (igual que contabo/auth.rs;
/// reqwest aquí no expone `.form()`).
fn pct(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' '                                                                => out.push('+'),
            b                                                                   => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

async fn access_token(http: &Client, cfg: &OutlookCfg) -> Result<String, String> {
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        cfg.tenant
    );
    let body = format!(
        "client_id={}&client_secret={}&scope={}&grant_type=client_credentials",
        pct(&cfg.client_id),
        pct(&cfg.client_secret),
        pct("https://graph.microsoft.com/.default"),
    );
    let res = http
        .post(url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send().await
        .map_err(|e| format!("outlook token request: {e}"))?;
    if !res.status().is_success() {
        let s = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(format!("outlook token {s}: {b}"));
    }
    Ok(res.json::<TokenResp>().await.map_err(|e| format!("outlook token decode: {e}"))?.access_token)
}

/// Base64 estándar (con padding) para `contentBytes` de adjuntos de Graph.
/// Inline para no agregar dependencia (el build es offline).
fn base64_encode(data: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for c in data.chunks(3) {
        let b1 = *c.get(1).unwrap_or(&0);
        let b2 = *c.get(2).unwrap_or(&0);
        let n = ((c[0] as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(A[((n >> 18) & 63) as usize] as char);
        out.push(A[((n >> 12) & 63) as usize] as char);
        out.push(if c.len() > 1 { A[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if c.len() > 2 { A[(n & 63) as usize] as char } else { '=' });
    }
    out
}

/// Envía a `to` la plantilla `clave` del JSON, sustituyendo `vars` (`{{k}}` → valor).
/// Fija un nombre de remitente legible (`OUTLOOK_SENDER_NAME`, default "Soflib") y
/// adjunta el archivo declarado en la plantilla (`attachment`) si se puede leer.
pub async fn send_template(to: &str, clave: &str, vars: &[(&str, &str)]) -> Result<(), String> {
    let cfg = OutlookCfg::from_env()?;
    let tpl = templates::load(clave)?;
    let subject = templates::render(&tpl.subject, vars);
    let html    = templates::render(&tpl.html, vars);
    let sender_name = std::env::var("OUTLOOK_SENDER_NAME")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Soflib".to_string());

    let mut message = json!({
        "subject": subject,
        "body": { "contentType": "HTML", "content": html },
        "from": { "emailAddress": { "address": cfg.sender.clone(), "name": sender_name } },
        "toRecipients": [ { "emailAddress": { "address": to } } ],
    });

    // Adjunto opcional de la plantilla (best-effort: si no se puede leer, va sin él).
    if let Some(path) = tpl.attachment.as_deref().filter(|p| !p.trim().is_empty()) {
        match std::fs::read(path) {
            Ok(bytes) => {
                let name = std::path::Path::new(path)
                    .file_name().and_then(|n| n.to_str()).unwrap_or("adjunto").to_string();
                let ctype = if name.to_lowercase().ends_with(".pdf") {
                    "application/pdf"
                } else {
                    "application/octet-stream"
                };
                message["attachments"] = json!([{
                    "@odata.type":  "#microsoft.graph.fileAttachment",
                    "name":         name,
                    "contentType":  ctype,
                    "contentBytes": base64_encode(&bytes),
                }]);
            }
            Err(e) => tracing::warn!(path, error = %e, "outlook: adjunto no leído; se envía sin él"),
        }
    }

    let http  = Client::new();
    let token = access_token(&http, &cfg).await?;
    let body  = json!({ "message": message });

    let res = http
        .post(format!("https://graph.microsoft.com/v1.0/users/{}/sendMail", cfg.sender))
        .bearer_auth(&token)
        .json(&body)
        .send().await
        .map_err(|e| format!("outlook sendMail request: {e}"))?;
    if !res.status().is_success() {
        let s = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(format!("outlook sendMail {s}: {b}"));
    }
    Ok(())
}
