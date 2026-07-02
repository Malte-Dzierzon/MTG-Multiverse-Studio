//! Card Service
//! 
//! 3-tier caching service for card data:
//! 1. Check SQLite (fastest, offline-capable)
//! 2. Check LRU cache (in-memory)
//! 3. Fetch from Scryfall API (rate-limited)

use crate::db::card_repo;
use crate::models::{CardDb, CardResponse};
use crate::scryfall::client::ScryfallClient;
use crate::utils::error::Result;
use serde_json::json;
/// Search cards — first in local DB, then optionally in Scryfall
pub fn search_cards_local(
    db: &rusqlite::Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<CardResponse>> {
    // Try FTS5 full-text search first
    let cards_db = match card_repo::search_cards_fts(db, query, limit) {
        Ok(cards) if !cards.is_empty() => cards,
        _ => {
            // Fallback: LIKE search
            card_repo::search_cards_by_name(db, query, Some(limit))?
        }
    };
    Ok(cards_db
        .iter()
        .map(|c| card_repo::card_db_to_response_with_conn(db, c))
        .collect())
}

/// Get a card by ID, trying DB first, then Scryfall API
pub async fn get_card(
    db: &rusqlite::Connection,
    scryfall: &ScryfallClient,
    id: &str,
) -> Result<CardResponse> {
    // Tier 1: Check local SQLite
    if let Some(card_db) = card_repo::get_card_by_id(db, id)? {
        return Ok(card_repo::card_db_to_response_with_conn(db, &card_db));
    }

    // Tier 2: Fetch from Scryfall API
    let api_card = scryfall.get_card_by_id(id).await?;
    
    // Convert to CardDb and store in SQLite
    let card_db = scryfall_card_to_carddb(&api_card, &api_card.set);
    card_repo::insert_card(db, &card_db)?;
    
    // Also store the set
    if let Err(e) = db.execute(
        "INSERT OR IGNORE INTO sets (id, name, set_type) VALUES (?1, ?2, ?3)",
        rusqlite::params![api_card.set, api_card.set_name, api_card.set_type],
    ) {
        tracing::warn!("Failed to cache set info: {}", e);
    }

    Ok(card_repo::card_db_to_response_with_conn(db, &card_db))
}

/// Search cards — try local first, fallback to Scryfall API
pub async fn search_cards(
    db: &rusqlite::Connection,
    scryfall: &ScryfallClient,
    query: &str,
    limit: usize,
) -> Result<Vec<CardResponse>> {
    // Tier 1: Search local SQLite
    let local = search_cards_local(db, query, limit)?;
    if !local.is_empty() {
        return Ok(local);
    }

    // Tier 2: Search Scryfall API
    let api_cards = scryfall.search_cards(query, None).await?;
    
    // Store results in SQLite and convert to response
    let mut results = Vec::with_capacity(api_cards.len().min(limit));
    for api_card in api_cards.iter().take(limit) {
        let card_db = scryfall_card_to_carddb(api_card, &api_card.set);
        let _ = card_repo::insert_card(db, &card_db);
        results.push(card_repo::card_db_to_response_with_conn(db, &card_db));
    }

    Ok(results)
}

/// Convert a Scryfall API card into our internal CardDb format
pub fn scryfall_card_to_carddb(
    api_card: &crate::scryfall::models::ScryfallCard,
    set_id: &str,
) -> CardDb {
    let image_uris_json = api_card
        .image_uris
        .as_ref()
        .map(|img| {
            json!({
                "small": img.small,
                "normal": img.normal,
                "large": img.large,
                "png": img.png,
                "art_crop": img.art_crop,
                "border_crop": img.border_crop,
            })
            .to_string()
        })
        .unwrap_or_else(|| "{}".to_string());

    let legalities_json = api_card
        .legalities
        .as_ref()
        .map(|leg| {
            json!({
                "standard": leg.standard,
                "modern": leg.modern,
                "legacy": leg.legacy,
                "vintage": leg.vintage,
                "commander": leg.commander,
                "pioneer": leg.pioneer,
                "pauper": leg.pauper,
            })
            .to_string()
        })
        .unwrap_or_else(|| "{}".to_string());

    let prices_json = api_card
        .prices
        .as_ref()
        .map(|p| {
            json!({
                "usd": p.usd,
                "usd_foil": p.usd_foil,
                "usd_etched": p.usd_etched,
                "eur": p.eur,
                "eur_foil": p.eur_foil,
                "eur_etched": p.eur_etched,
                "tix": p.tix,
            })
            .to_string()
        })
        .unwrap_or_else(|| "{}".to_string());

    CardDb {
        id: api_card.id.clone(),
        oracle_id: api_card.oracle_id.clone(),
        name: api_card.name.clone(),
        mana_cost: api_card.mana_cost.clone(),
        cmc: api_card.cmc,
        type_line: api_card.type_line.clone(),
        oracle_text: api_card.oracle_text.clone(),
        colors: json!(api_card.colors.iter().map(|c| c.to_string()).collect::<Vec<_>>()),
        color_identity: json!(api_card.color_identity.iter().map(|c| c.to_string()).collect::<Vec<_>>()),
        keywords: json!(api_card.keywords),
        rarity: api_card.rarity.clone(),
        set_id: set_id.to_string(),
        image_uris_json,
        artist: Some(api_card.artist.clone()),
        legalities: legalities_json,
        prices: prices_json,
    }
}
