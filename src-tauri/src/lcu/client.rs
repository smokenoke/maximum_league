//! HTTP client for LCU API (localhost, self-signed cert, Basic auth).

use crate::lcu::credentials::LcuCredentials;
use base64::Engine;
use reqwest::Client;
use std::time::Duration;

/// Build a reqwest client that accepts the LCU self-signed certificate (localhost only).
fn lcu_http_client() -> reqwest::Result<Client> {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5))
        .build()
}

/// Perform GET request to LCU API. Returns (status, body text).
pub async fn lcu_get(creds: &LcuCredentials, path: &str) -> Result<(u16, String), String> {
    let client = lcu_http_client().map_err(|e| e.to_string())?;
    let url = format!("https://127.0.0.1:{}{}", creds.port, path);
    let auth = format!("riot:{}", creds.password);
    let b64 = base64::engine::general_purpose::STANDARD.encode(auth.as_bytes());
    let response = client
        .get(&url)
        .header("Authorization", format!("Basic {}", b64))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status().as_u16();
    let body = response.text().await.map_err(|e| e.to_string())?;
    Ok((status, body))
}

/// Fetch a public URL (e.g. Community Dragon) with default TLS.
pub async fn fetch_public_url(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    response.text().await.map_err(|e| e.to_string())
}
