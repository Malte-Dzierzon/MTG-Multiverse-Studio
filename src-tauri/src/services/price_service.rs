//! Price Service
//!
//! Provides functions to refresh card prices from Scryfall and other sources.
//! Scryfall is the primary price source since it requires no API keys.

use crate::db::card_repo;
use crate::models::PriceRefreshResult;
use crate::scryfall::client::ScryfallClient;
use crate::utils::error::Result;
use rusqlite::Connection;
use serde_json;

/// Refresh prices for all cards in the database via Scryfall API.
///
/// Processes cards in batches of 50 (Scryfall's collection endpoint limit)
/// and updates the `prices` column for each card.
pub async fn refresh_all_prices(
    db: &Connection,
    client: &ScryfallClient,
) -> Result<PriceRefreshResult> {
    // Get all card IDs from the database
    let card_ids = card_repo::get_all_card_ids(db)?;
    let total = card_ids.len() as u64;

    if total == 0 {
        return Ok(PriceRefreshResult {
            total: 0,
            updated: 0,
            failed: 0,
            errors: vec![],
        });
    }

    let mut updated = 0u64;
    let mut failed = 0u64;
    let mut errors = Vec::new();

    // Process in batches of 50 (Scryfall /cards/collection limit is 75)
    for chunk in card_ids.chunks(50) {
        // Fetch batch from Scryfall
        match client.get_cards_by_collection(chunk).await {
            Ok(cards) => {
                for card in &cards {
                    // Extract prices from the Scryfall response
                    let prices_json = extract_prices_json(card);

                    // Update the database
                    if let Err(e) = card_repo::update_card_prices(db, &card.id, &prices_json) {
                        failed += 1;
                        errors.push(format!("DB update failed for {}: {}", card.name, e));
                    } else {
                        updated += 1;
                    }
                }
                // Cards that weren't found in Scryfall's response count as failed
                let batch_expected = chunk.len();
                let batch_found = cards.len();
                if batch_found < batch_expected {
                    let missing = batch_expected - batch_found;
                    failed += missing as u64;
                }
            }
            Err(e) => {
                failed += chunk.len() as u64;
                errors.push(format!(
                    "Scryfall batch fetch failed for chunk ({} cards): {}",
                    chunk.len(),
                    e
                ));
            }
        }

        // Small delay between batches to be polite to the API
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(PriceRefreshResult {
        total,
        updated,
        failed,
        errors,
    })
}

/// Refresh the price for a single card by its Scryfall ID.
pub async fn refresh_single_price(
    db: &Connection,
    card_id: &str,
    client: &ScryfallClient,
) -> Result<()> {
    let card = client.get_card_by_id(card_id).await?;
    let prices_json = extract_prices_json(&card);
    card_repo::update_card_prices(db, card_id, &prices_json)?;
    Ok(())
}

/// Extract a JSON prices object from a ScryfallCard
fn extract_prices_json(card: &crate::scryfall::models::ScryfallCard) -> serde_json::Value {
    match &card.prices {
        Some(prices) => {
            serde_json::json!({
                "usd": prices.usd,
                "usd_foil": prices.usd_foil,
                "eur": prices.eur,
                "eur_foil": prices.eur_foil,
                "tix": prices.tix,
            })
        }
        None => serde_json::json!({}),
    }
}
