//! Card Repository
//!
//! Handles database operations for the `cards` table.

use crate::models::{CardDb, CardLegalitiesResponse, CardPricesResponse, CardResponse};
use rusqlite::{named_params, params, Result};
use serde_json;

/// Insert a new card into the database
pub fn insert_card(conn: &rusqlite::Connection, card: &CardDb) -> Result<usize> {
    conn.execute(
        r#"
        INSERT INTO cards (
            id, oracle_id, name, mana_cost, cmc, type_line, oracle_text,
            colors, color_identity, keywords, rarity, set_id, set_code, image_uris_json, artist,
            legalities, prices
        ) VALUES (
            :id, :oracle_id, :name, :mana_cost, :cmc, :type_line, :oracle_text,
            :colors, :color_identity, :keywords, :rarity, :set_id, :set_code, :image_uris, :artist,
            :legalities, :prices
        )
        "#,
        named_params! {
            ":id": &card.id,
            ":oracle_id": &card.oracle_id,
            ":name": &card.name,
            ":mana_cost": &card.mana_cost,
            ":cmc": card.cmc,
            ":type_line": &card.type_line,
            ":oracle_text": &card.oracle_text,
            ":colors": serde_json::to_string(&card.colors).unwrap_or_else(|_| "[]".into()),
            ":color_identity": serde_json::to_string(&card.color_identity).unwrap_or_else(|_| "[]".into()),
            ":keywords": serde_json::to_string(&card.keywords).unwrap_or_else(|_| "[]".into()),
            ":rarity": &card.rarity,
            ":set_id": &card.set_id,
            ":set_code": &card.set_code,
            ":image_uris_json": &card.image_uris_json,
            ":artist": &card.artist,
            ":legalities": &card.legalities,
            ":prices": &card.prices,
        },
    )
}

/// Get a card by its Scryfall ID
pub fn get_card_by_id(conn: &rusqlite::Connection, id: &str) -> Result<Option<CardDb>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            id, oracle_id, name, mana_cost, cmc, type_line, oracle_text,
            colors, color_identity, keywords, rarity, set_id, set_code, image_uris_json, artist,
            legalities, prices
        FROM cards WHERE id = :id
        "#,
    )?;

    let mut rows = stmt.query_map(named_params! { ":id": id }, |row| CardDb::from_row(row))?;

    match rows.next() {
        Some(Ok(card)) => Ok(Some(card)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// Search cards by name (case-insensitive, LIKE match)
pub fn search_cards_by_name(
    conn: &rusqlite::Connection,
    name: &str,
    limit: Option<usize>,
) -> Result<Vec<CardDb>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            id, oracle_id, name, mana_cost, cmc, type_line, oracle_text,
            colors, color_identity, keywords, rarity, set_id, set_code, image_uris_json, artist,
            legalities, prices
        FROM cards WHERE name LIKE :name
        LIMIT :limit
        "#,
    )?;

    let like_pattern = format!("%{}%", name);
    let limit_val = limit.unwrap_or(200) as i64;

    let rows = stmt.query_map(
        named_params! {
            ":name": like_pattern,
            ":limit": limit_val,
        },
        |row| CardDb::from_row(row),
    )?;

    let mut cards = Vec::new();
    for row in rows {
        cards.push(row?);
    }

    Ok(cards)
}
/// Search cards using FTS5 full-text search
pub fn search_cards_fts(
    conn: &rusqlite::Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<CardDb>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            c.id, c.oracle_id, c.name, c.mana_cost, c.cmc, c.type_line, c.oracle_text,
            c.colors, c.color_identity, c.keywords, c.rarity, c.set_id, c.set_code, c.image_uris_json, c.artist,
            c.legalities, c.prices
        FROM cards_fts f
        JOIN cards c ON c.rowid = f.rowid
        WHERE cards_fts MATCH :query
        ORDER BY rank
        LIMIT :limit
        "#,
    )?;

    let limit_val = limit as i64;

    let rows = stmt.query_map(
        named_params! {
            ":query": query,
            ":limit": limit_val,
        },
        |row| CardDb::from_row(row),
    )?;

    let mut cards = Vec::new();
    for row in rows {
        cards.push(row?);
    }

    Ok(cards)
}
/// Count total cards in the database
pub fn count_cards(conn: &rusqlite::Connection) -> Result<u64> {
    let count: u64 = conn.query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))?;
    Ok(count)
}

/// Convert a CardDb (internal DB representation) to CardResponse (frontend)
pub fn card_db_to_response(card: &CardDb) -> CardResponse {
    base_card_to_response(card, None)
}

/// Convert a CardDb to CardResponse, looking up the set name from the sets table
pub fn card_db_to_response_with_conn(
    conn: &rusqlite::Connection,
    card: &CardDb,
) -> CardResponse {
    base_card_to_response(card, Some(conn))
}

/// Core conversion logic, optionally resolving set_name from DB
fn base_card_to_response(card: &CardDb, conn: Option<&rusqlite::Connection>) -> CardResponse {
    let colors: Vec<String> =
        serde_json::from_value(card.colors.clone()).unwrap_or_default();
    let color_identity: Vec<String> =
        serde_json::from_value(card.color_identity.clone()).unwrap_or_default();
    let keywords: Vec<String> =
        serde_json::from_value(card.keywords.clone()).unwrap_or_default();
    let legalities: serde_json::Value =
        serde_json::from_str(&card.legalities).unwrap_or_else(|_| serde_json::json!({}));
    let prices: serde_json::Value =
        serde_json::from_str(&card.prices).unwrap_or_else(|_| serde_json::json!({}));

    let image_url_small = serde_json::from_str::<serde_json::Value>(&card.image_uris_json)
        .ok()
        .and_then(|v| {
            v.get("small")
                .and_then(|s| s.as_str())
                .map(String::from)
        });
    let image_url_large = serde_json::from_str::<serde_json::Value>(&card.image_uris_json)
        .ok()
        .and_then(|v| {
            v.get("large")
                .and_then(|s| s.as_str())
                .map(String::from)
        });

    let get_price = |key: &str| -> Option<String> {
        prices
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    };

    let get_legality = |key: &str| -> String {
        legalities
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("not_legal")
            .to_string()
    };

    // Look up set name if connection is available
    let set_name = conn
        .and_then(|c| {
            c.query_row(
                "SELECT name FROM sets WHERE id = ?1",
                params![card.set_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
        .unwrap_or_default();

    CardResponse {
        id: card.id.clone(),
        name: card.name.clone(),
        mana_cost: card.mana_cost.clone(),
        cmc: card.cmc,
        type_line: card.type_line.clone(),
        oracle_text: card.oracle_text.clone(),
        colors,
        color_identity,
        keywords,
        rarity: card.rarity.clone(),
        set: card.set_id.clone(),
        set_name,
        artist: card.artist.clone(),
        image_url_small,
        image_url_large,
        prices: Some(CardPricesResponse {
            usd: get_price("usd"),
            usd_foil: get_price("usd_foil"),
            eur: get_price("eur"),
            tix: get_price("tix"),
        }),
        legalities: Some(CardLegalitiesResponse {
            standard: get_legality("standard"),
            modern: get_legality("modern"),
            legacy: get_legality("legacy"),
            vintage: get_legality("vintage"),
            commander: get_legality("commander"),
            pioneer: get_legality("pioneer"),
            pauper: get_legality("pauper"),
        }),
    }
}

/// Get all card IDs from the database
pub fn get_all_card_ids(conn: &rusqlite::Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM cards")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let ids: Vec<String> = rows.filter_map(|r| r.ok()).collect();
    Ok(ids)
}

/// Update the prices JSON for a specific card
pub fn update_card_prices(
    conn: &rusqlite::Connection,
    card_id: &str,
    prices: &serde_json::Value,
) -> Result<usize> {
    let prices_str = serde_json::to_string(prices).unwrap_or_else(|_| "{}".to_string());
    conn.execute(
        "UPDATE cards SET prices = ?1 WHERE id = ?2",
        rusqlite::params![prices_str, card_id],
    )
}

/// Insert a price history entry for a card
pub fn insert_price_history(
    conn: &rusqlite::Connection,
    card_id: &str,
    source: &str,
    currency: &str,
    price_low: Option<f64>,
    price_avg: Option<f64>,
    price_high: Option<f64>,
    price_trend: Option<f64>,
) -> Result<usize> {
    conn.execute(
        r#"
        INSERT INTO price_history (card_id, source, currency, price_low, price_avg, price_high, price_trend)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        rusqlite::params![card_id, source, currency, price_low, price_avg, price_high, price_trend],
    )
}
