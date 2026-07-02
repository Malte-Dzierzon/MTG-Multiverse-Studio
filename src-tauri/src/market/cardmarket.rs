//! Cardmarket (MKM) API Client
//!
//! Uses OAuth 1.0a to authenticate against the Cardmarket API v2.0.
//! Requires CARDMARKET_APP_KEY and CARDMARKET_APP_SECRET environment variables.
//! Gracefully degrades (returns None) when keys are not set.
//!
//! API docs: https://api.cardmarket.com/ws/v2.0/output.json/

use crate::utils::error::{AppError, Result};
use base64::Engine;
use hmac::{Hmac, Mac};
use rand::Rng;
use reqwest::Client;
use sha1::Sha1;
use std::collections::BTreeMap;

/// A market price entry from Cardmarket
#[derive(Debug, Clone)]
pub struct MarketPrice {
    pub card_name: String,
    pub set_name: String,
    pub currency: String,
    pub low: Option<f64>,
    pub avg: Option<f64>,
    pub high: Option<f64>,
    pub trend: Option<f64>,
}

/// Cardmarket API client with OAuth 1.0a authentication
pub struct CardmarketClient {
    http_client: Client,
    base_url: &'static str,
    app_key: Option<String>,
    app_secret: Option<String>,
    access_token: Option<String>,
    access_token_secret: Option<String>,
}

impl CardmarketClient {
    /// Create a new Cardmarket client from environment variables.
    ///
    /// Reads:
    /// - `CARDMARKET_APP_KEY` (required for operation)
    /// - `CARDMARKET_APP_SECRET` (required for operation)
    /// - `CARDMARKET_ACCESS_TOKEN` (optional)
    /// - `CARDMARKET_ACCESS_TOKEN_SECRET` (optional)
    pub fn new() -> Self {
        let app_key = std::env::var("CARDMARKET_APP_KEY").ok();
        let app_secret = std::env::var("CARDMARKET_APP_SECRET").ok();
        let access_token = std::env::var("CARDMARKET_ACCESS_TOKEN").ok();
        let access_token_secret = std::env::var("CARDMARKET_ACCESS_TOKEN_SECRET").ok();

        Self {
            http_client: Client::builder()
                .user_agent("mtg-multiverse-studio/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
            base_url: "https://api.cardmarket.com/ws/v2.0/output.json",
            app_key,
            app_secret,
            access_token,
            access_token_secret,
        }
    }

    /// Whether the client has been configured with API keys
    pub fn is_configured(&self) -> bool {
        self.app_key.is_some() && self.app_secret.is_some()
    }

    /// Fetch the market price for a card by name and set code.
    ///
    /// Returns `Ok(None)` when:
    /// - The client is not configured (no API keys)
    /// - The card is not found on Cardmarket
    /// - The API request fails for a recoverable reason
    pub async fn get_price_for_card(
        &self,
        card_name: &str,
        set_code: &str,
    ) -> Result<Option<MarketPrice>> {
        if !self.is_configured() {
            return Ok(None);
        }

        // Cardmarket search endpoint
        let url = format!(
            "{}/articles/search?search={}&idGame=1&idLanguage=1",
            self.base_url,
            percent_encoding::utf8_percent_encode(card_name, percent_encoding::NON_ALPHANUMERIC)
        );

        let oauth_header = self.build_oauth_header("GET", &url)?;

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", oauth_header)
            .send()
            .await
            .map_err(|e| AppError::Http(format!("Cardmarket request failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(None); // Graceful degradation on non-200
        }

        // Parse response (Cardmarket returns JSON with 'article' array)
        let body: serde_json::Value = response.json().await.map_err(|e| {
            AppError::Json(format!("Cardmarket parse error: {}", e))
        })?;

        let price = self.extract_price(&body, set_code);
        Ok(price)
    }

    /// Extract the most relevant price from Cardmarket's response JSON
    fn extract_price(
        &self,
        body: &serde_json::Value,
        _set_code: &str,
    ) -> Option<MarketPrice> {
        let articles = body.get("article")?.as_array()?;

        if articles.is_empty() {
            return None;
        }

        let mut price_low_sum = 0.0f64;
        let mut price_avg_sum = 0.0f64;
        let mut count = 0u32;

        for article in articles {
            if let Some(price) = article.get("price").and_then(|p| p.as_f64()) {
                price_avg_sum += price;
                if price_low_sum == 0.0 || price < price_low_sum {
                    price_low_sum = price;
                }
                count += 1;
            }
        }

        if count == 0 {
            return None;
        }

        let avg = price_avg_sum / count as f64;

        Some(MarketPrice {
            card_name: body
                .get("product")
                .and_then(|p| p.get("enName"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            set_name: body
                .get("product")
                .and_then(|p| p.get("expansionName"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            currency: "EUR".to_string(),
            low: Some(price_low_sum),
            avg: Some(avg),
            high: None,
            trend: None,
        })
    }

    /// Build an OAuth 1.0a Authorization header
    fn build_oauth_header(&self, method: &str, url: &str) -> Result<String> {
        let app_key = self
            .app_key
            .as_ref()
            .ok_or_else(|| AppError::Unknown("CARDMARKET_APP_KEY not set".into()))?;
        let app_secret = self
            .app_secret
            .as_ref()
            .ok_or_else(|| AppError::Unknown("CARDMARKET_APP_SECRET not set".into()))?;

        let nonce = generate_nonce();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let mut params = BTreeMap::new();
        params.insert("oauth_consumer_key", app_key.as_str());
        params.insert("oauth_nonce", nonce.as_str());
        params.insert("oauth_signature_method", "HMAC-SHA1");
        params.insert("oauth_timestamp", timestamp.as_str());
        params.insert("oauth_version", "1.0");

        if let Some(ref token) = self.access_token {
            params.insert("oauth_token", token.as_str());
        }

        // Build signature base string
        let param_string: String = params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    percent_encoding::utf8_percent_encode(k, percent_encoding::NON_ALPHANUMERIC),
                    percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC)
                )
            })
            .collect::<Vec<_>>()
            .join("&");

        let signature_base = format!(
            "{}&{}&{}",
            method,
            percent_encoding::utf8_percent_encode(url, percent_encoding::NON_ALPHANUMERIC),
            percent_encoding::utf8_percent_encode(&param_string, percent_encoding::NON_ALPHANUMERIC)
        );

        // Sign with HMAC-SHA1
        let consumer_secret_enc = percent_encoding::utf8_percent_encode(
            app_secret,
            percent_encoding::NON_ALPHANUMERIC,
        );
        let token_secret_enc = match self.access_token_secret {
            Some(ref s) => percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string(),
            None => String::new(),
        };
        let signing_key = format!("{}&{}", consumer_secret_enc, token_secret_enc);

        let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes())
            .map_err(|e| AppError::Unknown(format!("HMAC init error: {}", e)))?;
        mac.update(signature_base.as_bytes());
        let signature = mac.finalize().into_bytes();
        let signature_b64 = base64::engine::general_purpose::STANDARD.encode(&signature);

        // Build Authorization header
        let mut auth_parts: Vec<String> = params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}=\"{}\"",
                    k,
                    percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC)
                )
            })
            .collect();
        auth_parts.push(format!(
            "oauth_signature=\"{}\"",
            percent_encoding::utf8_percent_encode(
                &signature_b64,
                percent_encoding::NON_ALPHANUMERIC
            )
        ));

        Ok(format!("OAuth {}", auth_parts.join(", ")))
    }
}

impl Clone for CardmarketClient {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            base_url: self.base_url,
            app_key: self.app_key.clone(),
            app_secret: self.app_secret.clone(),
            access_token: self.access_token.clone(),
            access_token_secret: self.access_token_secret.clone(),
        }
    }
}

/// Generate a random nonce for OAuth 1.0a
fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    (0..32).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}
