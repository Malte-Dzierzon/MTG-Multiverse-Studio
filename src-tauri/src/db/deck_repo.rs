//! Deck Repository
//! 
//! Handles database operations for the `decks` and `deck_cards` tables.

use rusqlite::{named_params, params, Result};
use serde::{Deserialize, Serialize};

/// Deck metadata from SQLite
#[derive(Debug, Clone)]
pub struct DeckDb {
    pub id: i64,
    pub name: String,
    pub format: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// A single card in a deck (join row)
#[derive(Debug, Clone)]
pub struct DeckCardDb {
    pub deck_id: i64,
    pub card_id: String,
    pub quantity: i32,
    pub position: i32,
    pub category: String,
}

/// A combined deck row with its card join data (for batch queries)
#[derive(Debug, Clone)]
pub struct DeckWithCardRow {
    pub deck_id: i64,
    pub deck_name: String,
    pub deck_format: Option<String>,
    pub deck_description: Option<String>,
    pub deck_created_at: String,
    pub deck_updated_at: Option<String>,
    pub card_id: Option<String>,
    pub quantity: Option<i32>,
    pub position: Option<i32>,
    pub category: Option<String>,
}

/// Create a new deck
pub fn create_deck(
    conn: &rusqlite::Connection,
    name: &str,
    format: Option<&str>,
    description: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO decks (name, format, description, updated_at)
         VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
        params![name, format, description],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get a single deck by ID
pub fn get_deck_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Option<DeckDb>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, format, description, created_at, updated_at
         FROM decks WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        Ok(DeckDb {
            id: row.get(0)?,
            name: row.get(1)?,
            format: row.get(2)?,
            description: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    match rows.next() {
        Some(Ok(deck)) => Ok(Some(deck)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// Get all decks, most recently updated first
pub fn get_all_decks(conn: &rusqlite::Connection) -> Result<Vec<DeckDb>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, format, description, created_at, updated_at
         FROM decks ORDER BY updated_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(DeckDb {
            id: row.get(0)?,
            name: row.get(1)?,
            format: row.get(2)?,
            description: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    let mut decks = Vec::new();
    for row in rows {
        decks.push(row?);
    }
    Ok(decks)
}

/// Update deck metadata
pub fn update_deck(
    conn: &rusqlite::Connection,
    id: i64,
    name: &str,
    format: Option<&str>,
    description: Option<&str>,
) -> Result<usize> {
    conn.execute(
        "UPDATE decks SET name = ?1, format = ?2, description = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
        params![name, format, description, id],
    )
}

/// Delete a deck and all its card associations (CASCADE)
pub fn delete_deck(conn: &rusqlite::Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM decks WHERE id = ?1", params![id])
}

/// Search decks by name (case-insensitive)
pub fn search_decks_by_name(
    conn: &rusqlite::Connection,
    query: &str,
) -> Result<Vec<DeckDb>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, format, description, created_at, updated_at
         FROM decks WHERE name LIKE ?1 ORDER BY updated_at DESC",
    )?;

    let pattern = format!("%{}%", query);
    let rows = stmt.query_map(params![pattern], |row| {
        Ok(DeckDb {
            id: row.get(0)?,
            name: row.get(1)?,
            format: row.get(2)?,
            description: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    let mut decks = Vec::new();
    for row in rows {
        decks.push(row?);
    }
    Ok(decks)
}

/// Get the color identity of a deck (distinct colors across all cards)
pub fn get_deck_colors(
    conn: &rusqlite::Connection,
    deck_id: i64,
) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT DISTINCT json_each.value
        FROM deck_cards dc
        JOIN cards c ON c.id = dc.card_id
        CROSS JOIN json_each(c.color_identity)
        WHERE dc.deck_id = ?1
        ORDER BY json_each.value
        "#,
    )?;

    let rows = stmt.query_map(params![deck_id], |row| row.get::<_, String>(0))?;

    let mut colors = Vec::new();
    for row in rows {
        colors.push(row?);
    }
    Ok(colors)
}

/// Validate deck legality against a format.
/// Returns a list of card names that are NOT legal in the given format.
pub fn validate_deck_legality(
    conn: &rusqlite::Connection,
    deck_id: i64,
    format: &str,
) -> Result<Vec<String>> {
    // Query: check the legalities JSON for each card in the deck
    let mut stmt = conn.prepare(
        r#"
        SELECT c.name
        FROM deck_cards dc
        JOIN cards c ON c.id = dc.card_id
        WHERE dc.deck_id = ?1
          AND (
            json_extract(c.legalities, '$.' || ?2) IS NULL
            OR json_extract(c.legalities, '$.' || ?2) = 'not_legal'
          )
        ORDER BY c.name
        "#,
    )?;

    let rows = stmt.query_map(params![deck_id, format], |row| row.get::<_, String>(0))?;

    let mut illegal = Vec::new();
    for row in rows {
        illegal.push(row?);
    }
    Ok(illegal)
}

// ─── DECK_CARDS ──────────────────────────────────

/// Add a card to a deck (upsert — increments quantity if already present)
pub fn add_card_to_deck(
    conn: &rusqlite::Connection,
    deck_id: i64,
    card_id: &str,
    quantity: i32,
    position: i32,
    category: &str,
) -> Result<usize> {
    conn.execute(
        "INSERT INTO deck_cards (deck_id, card_id, quantity, position, category)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(deck_id, card_id) DO UPDATE SET
           quantity = quantity + ?3,
           category = ?5",
        params![deck_id, card_id, quantity, position, category],
    )
}

/// Remove a card from a deck (deletes the row entirely)
pub fn remove_card_from_deck(
    conn: &rusqlite::Connection,
    deck_id: i64,
    card_id: &str,
) -> Result<usize> {
    conn.execute(
        "DELETE FROM deck_cards WHERE deck_id = ?1 AND card_id = ?2",
        params![deck_id, card_id],
    )
}

/// Get all cards in a deck
pub fn get_deck_cards(
    conn: &rusqlite::Connection,
    deck_id: i64,
) -> Result<Vec<DeckCardDb>> {
    let mut stmt = conn.prepare(
        "SELECT deck_id, card_id, quantity, position, category
         FROM deck_cards WHERE deck_id = ?1 ORDER BY position, card_id",
    )?;

    let rows = stmt.query_map(params![deck_id], |row| {
        Ok(DeckCardDb {
            deck_id: row.get(0)?,
            card_id: row.get(1)?,
            quantity: row.get(2)?,
            position: row.get(3)?,
            category: row.get::<_, Option<String>>(4)?
                .unwrap_or_else(|| "mainboard".to_string()),
        })
    })?;

    let mut cards = Vec::new();
    for row in rows {
        cards.push(row?);
    }
    Ok(cards)
}

/// Count total cards in a deck (sum of quantities)
pub fn count_deck_cards(
    conn: &rusqlite::Connection,
    deck_id: i64,
) -> Result<i32> {
    conn.query_row(
        "SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = ?1",
        params![deck_id],
        |row| row.get(0),
    )
}

/// Get all decks with their cards in a single JOIN query (N+1 fix).
/// Returns Vec<DeckWithCardRow>, to be grouped in application code.
pub fn get_decks_with_cards_batch(
    conn: &rusqlite::Connection,
) -> Result<Vec<DeckWithCardRow>> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.name, d.format, d.description, d.created_at, d.updated_at,
                dc.card_id, dc.quantity, dc.position, dc.category
         FROM decks d
         LEFT JOIN deck_cards dc ON dc.deck_id = d.id
         ORDER BY d.updated_at DESC, d.id, dc.position",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(DeckWithCardRow {
            deck_id: row.get(0)?,
            deck_name: row.get(1)?,
            deck_format: row.get(2)?,
            deck_description: row.get(3)?,
            deck_created_at: row.get(4)?,
            deck_updated_at: row.get(5)?,
            card_id: row.get(6)?,
            quantity: row.get(7)?,
            position: row.get(8)?,
            category: row.get::<_, Option<String>>(9)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}
