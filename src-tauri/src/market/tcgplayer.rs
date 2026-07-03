//! TCGPlayer API Client
//!
//! Uses Bearer token authentication against the TCGPlayer API v1.37.
//! Requires TCGPLAYER_API_KEY environment variable (clients also need TCGPLAYER_API_SECRET).
//! Gracefully degrades (returns None) when keys are not set.
//!
//! API docs: https://docs.tcgplayer.com/docs

use crate::utils::error::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// A market price from TCGPlayer
#[derive(Debug, Clone)]
pub struct TcgplayerPrice {
    pub product_id: u32,
    pub product_name: String,
    pub low_price: Option<f64>,
    pub market_price: Option<f64>,
    pub mid_price: Option<f64>,
    pub high_price: Option<f64>,
    pub currency: String,
}

/// OAuth2 token response from TCGPlayer
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: Option<u32>,
    #[allow(dead_code)]
    token_type: Option<String>,
}

/// Product search response
#[derive(Debug, Deserialize)]
struct ProductSearchResponse {
    results: Option<Vec<ProductResult>>,
    total_items: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ProductResult {
    product_id: u32,
    name: String,
    #[serde(rename = "extendedData")]
    extended_data: Option<Vec<ExtendedData>>,
}

#[derive(Debug, Deserialize)]
struct ExtendedData {
    name: String,
    value: String,
}

/// Market price response
#[derive(Debug, Deserialize)]
struct MarketPriceResponse {
    results: Option<Vec<MarketPriceEntry>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MarketPriceEntry {
    #[serde(rename = "productId")]
    product_id: u32,
    #[serde(rename = "lowPrice")]
    low_price: Option<f64>,
    #[serde(rename = "marketPrice")]
    market_price: Option<f64>,
    #[serde(rename = "midPrice")]
    mid_price: Option<f64>,
    #[serde(rename = "highPrice")]
    high_price: Option<f64>,
    #[serde(rename = "directLowPrice")]
    direct_low_price: Option<f64>,
}

/// TCGPlayer API client
pub struct TcgplayerClient {
    http_client: Client,
    base_url: &'static str,
    api_key: Option<String>,
    api_secret: Option<String>,
    token: Option<String>,
}

impl Clone for TcgplayerClient {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            base_url: self.base_url,
            api_key: self.api_key.clone(),
            api_secret: self.api_secret.clone(),
            token: self.token.clone(),
        }
    }
}

impl TcgplayerClient {
    /// Create a new TCGPlayer client from environment variables.
    ///
    /// Reads:
    /// - `TCGPLAYER_API_KEY` (required for operation)
    /// - `TCGPLAYER_API_SECRET` (required for OAuth2 token)
    pub fn new() -> Self {
        let api_key = std::env::var("TCGPLAYER_API_KEY").ok();
        let api_secret = std::env::var("TCGPLAYER_API_SECRET").ok();

        Self {
            http_client: Client::builder()
                .user_agent("mtg-multiverse-studio/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
            base_url: "https://api.tcgplayer.com/v1.37",
            api_key,
            api_secret,
            token: None,
        }
    }

    /// Whether the client has been configured with API credentials
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    /// Authenticate and get an access token
    async fn ensure_token(&mut self) -> Result<()> {
        if self.token.is_some() {
            return Ok(());
        }

        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Unknown("TCGPLAYER_API_KEY not set".into()))?;
        let api_secret = self
            .api_secret
            .as_ref()
            .ok_or_else(|| AppError::Unknown("TCGPLAYER_API_SECRET not set".into()))?;

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", api_key),
            ("client_secret", api_secret),
        ];

        let response = self
            .http_client
            .post("https://api.tcgplayer.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Http(format!("TCGPlayer token request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Http(format!(
                "TCGPlayer token error: {}",
                response.status()
            )));
        }

        let token_resp: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::Json(format!("TCGPlayer token parse error: {}", e)))?;

        self.token = Some(token_resp.access_token);
        Ok(())
    }

    /// Search for a product by name
    async fn search_product(
        &self,
        card_name: &str,
        _set_code: &str,
    ) -> Result<Option<ProductResult>> {
        let token = match self.token {
            Some(ref t) => t,
            None => return Ok(None),
        };

        let url = format!(
            "{}/catalog/products?search={}&categoryId=1&limit=10&getExtendedFields=true",
            self.base_url,
            percent_encoding::utf8_percent_encode(card_name, percent_encoding::NON_ALPHANUMERIC)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::Http(format!("TCGPlayer search failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let search_resp: ProductSearchResponse = response
            .json()
            .await
            .map_err(|e| AppError::Json(format!("TCGPlayer search parse error: {}", e)))?;

        Ok(search_resp.results.and_then(|r| r.into_iter().next()))
    }

    /// Fetch market price for a specific product
    async fn get_market_price_for_product(
        &self,
        product_id: u32,
    ) -> Result<Option<TcgplayerPrice>> {
        let token = match self.token {
            Some(ref t) => t,
            None => return Ok(None),
        };

        let url = format!(
            "{}/pricing/product/{}",
            self.base_url, product_id
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AppError::Http(format!("TCGPlayer pricing failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        // TCGPlayer returns an array of price entries
        let prices_resp: Vec<MarketPriceEntry> = response
            .json()
            .await
            .map_err(|e| AppError::Json(format!("TCGPlayer pricing parse error: {}", e)))?;

        if let Some(entry) = prices_resp.into_iter().next() {
            Ok(Some(TcgplayerPrice {
                product_id,
                product_name: String::new(), // We'd need a separate product name lookup
                low_price: entry.low_price,
                market_price: entry.market_price,
                mid_price: entry.mid_price,
                high_price: entry.high_price,
                currency: "USD".to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get market price for a card by name and set code.
    ///
    /// Returns `Ok(None)` when:
    /// - The client is not configured
    /// - The card is not found
    /// - API request fails
    pub async fn get_market_price(
        &mut self,
        card_name: &str,
        set_code: &str,
    ) -> Result<Option<TcgplayerPrice>> {
        if !self.is_configured() {
            return Ok(None);
        }

        // Ensure we have a token
        self.ensure_token().await?;

        // Search for the product
        let product = match self.search_product(card_name, set_code).await? {
            Some(p) => p,
            None => return Ok(None),
        };

        // Get market price
        self.get_market_price_for_product(product.product_id).await
    }
}
