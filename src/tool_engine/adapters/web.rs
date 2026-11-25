//! Web adapter — per Architecture §13 (Tool Engine).
//! Provides web operations using reqwest when the http feature is enabled.

use serde_json;

/// Web adapter for HTTP operations.
#[derive(Default)]
pub struct WebAdapter;

impl WebAdapter {
    /// Create a new web adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "web",
            "adapted": true
        })
    }

    /// Fetch the contents of a URL.
    pub fn fetch(&self, url: &str) -> Result<String, String> {
        #[cfg(feature = "http")]
        {
            let client = reqwest::blocking::Client::new();
            let response = client
                .get(url)
                .send()
                .map_err(|e| format!("Failed to fetch {url}: {e}"))?;
            let text = response
                .text()
                .map_err(|e| format!("Failed to read response from {url}: {e}"))?;
            Ok(text)
        }
        #[cfg(not(feature = "http"))]
        {
            Err(format!(
                "Web adapter requires the 'http' cargo feature for URL: {url}"
            ))
        }
    }
}
