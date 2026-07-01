//! Scryfall API Client
//!
//! Handles HTTP requests to the Scryfall API with rate limiting and caching.

use crate::scryfall::models::*;
use crate::utils::error::{AppError, Result};
use lru::LruCache;
use reqwest::Client;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

/// Scryfall client with rate limiting and caching
pub struct ScryfallClient {
    http_client: Client,
    base_url: &'static str,
    rate_limiter: Arc<Semaphore>,
    cache: Arc<Mutex<LruCache<String, ScryfallCard>>>,
}

impl ScryfallClient {
    /// Create a new Scryfall client with default settings
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .user_agent("mtg-multiverse-studio/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
            base_url: "https://api.scryfall.com",
            rate_limiter: Arc::new(Semaphore::new(10)),
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(1000).expect("Non-zero usize"),
            ))),
        }
    }

    /// Get a card by its Scryfall ID
    pub async fn get_card_by_id(&self, id: &str) -> Result<ScryfallCard> {
        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(card) = cache.get(id) {
                return Ok(card.clone());
            }
        }

        // Rate limit
        let _permit = self.rate_limiter.acquire().await;

        let url = format!("{}/cards/{}", self.base_url, id);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::Unknown(format!(
                "Scryfall API error {} for card '{}'",
                response.status(),
                id
            )));
        }

        let card: ScryfallCard = response.json().await?;

        // Cache the result
        let mut cache = self.cache.lock().unwrap();
        cache.put(card.id.clone(), card.clone());

        Ok(card)
    }

    /// Get a card by its name (fuzzy search)
    pub async fn get_card_by_name(&self, name: &str) -> Result<ScryfallCard> {
        let _permit = self.rate_limiter.acquire().await;

        let url = format!(
            "{}/cards/named?fuzzy={}",
            self.base_url,
            urlencoding::encode(name)
        );
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::Unknown(format!(
                "Scryfall API error {} for card '{}'",
                response.status(),
                name
            )));
        }

        let card: ScryfallCard = response.json().await?;

        let mut cache = self.cache.lock().unwrap();
        cache.put(card.id.clone(), card.clone());

        Ok(card)
    }

    /// Search for cards by query
    pub async fn search_cards(
        &self,
        query: &str,
        page: Option<i32>,
    ) -> Result<Vec<ScryfallCard>> {
        let _permit = self.rate_limiter.acquire().await;

        let mut url = format!(
            "{}/cards/search?q={}",
            self.base_url,
            urlencoding::encode(query)
        );
        if let Some(page) = page {
            url.push_str(&format!("&page={}", page));
        }

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::Unknown(format!(
                "Scryfall search error {} for '{}'",
                response.status(),
                query
            )));
        }

        let search_result: ScryfallList = response.json().await?;
        Ok(search_result.data)
    }

    /// Get a set by code
    pub async fn get_set_by_code(&self, code: &str) -> Result<ScryfallSet> {
        let _permit = self.rate_limiter.acquire().await;

        let url = format!("{}/sets/{}", self.base_url, code);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::Unknown(format!(
                "Scryfall API error {} for set '{}'",
                response.status(),
                code
            )));
        }

        let set: ScryfallSet = response.json().await?;
        Ok(set)
    }
}

impl Clone for ScryfallClient {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            base_url: self.base_url,
            rate_limiter: self.rate_limiter.clone(),
            cache: self.cache.clone(),
        }
    }
}
