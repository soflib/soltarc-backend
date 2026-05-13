// OAuth2 password-grant token management for the Contabo management API.
// Tokens are cached until 60 s before expiry to avoid redundant round-trips.
//
// Uses a manual application/x-www-form-urlencoded body so no reqwest feature
// flags beyond the defaults are required.

use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};

const TOKEN_URL: &str =
    "https://auth.contabo.com/auth/realms/contabo/protocol/openid-connect/token";

// ── Token cache ───────────────────────────────────────────────────────────────

pub struct TokenState {
    value:      String,
    expires_at: Option<Instant>,
}

impl Default for TokenState {
    fn default() -> Self {
        Self { value: String::new(), expires_at: None }
    }
}

impl TokenState {
    pub fn is_valid(&self) -> bool {
        self.expires_at
            .map(|e| Instant::now() + Duration::from_secs(60) < e)
            .unwrap_or(false)
    }

    pub fn get(&self) -> &str { &self.value }

    pub fn set(&mut self, token: String, expires_in: u64) {
        self.value      = token;
        self.expires_at = Some(Instant::now() + Duration::from_secs(expires_in));
    }
}

// ── Token fetch ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TokenResp {
    access_token: String,
    expires_in:   u64,
}

/// Percent-encode a string for use in application/x-www-form-urlencoded bodies.
fn pct(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' '                         => out.push('+'),
            b                            => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub async fn fetch(
    http:    &Client,
    cid:     &str,
    csecret: &str,
    user:    &str,
    pass:    &str,
) -> Result<(String, u64), String> {
    let body = format!(
        "client_id={}&client_secret={}&username={}&password={}&grant_type=password",
        pct(cid), pct(csecret), pct(user), pct(pass)
    );

    let res = http
        .post(TOKEN_URL)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("contabo auth request: {e}"))?;

    if !res.status().is_success() {
        let s = res.status();
        let b = res.text().await.unwrap_or_default();
        return Err(format!("contabo auth {s}: {b}"));
    }

    let r: TokenResp = res.json().await
        .map_err(|e| format!("contabo auth decode: {e}"))?;

    Ok((r.access_token, r.expires_in))
}
