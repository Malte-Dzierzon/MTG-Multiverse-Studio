//! Collection Repository
//! 
//! Handles database operations for the `collection` table (user's card collection).
//! Supports CRUD, pagination, search, and batch import.

use crate::import_engine::collection_import::CollectionImportItem;
use crate::models::{CollectionItemDb, ImportStats};
use rusqlite::Result;

/// Add a card to collection (upsert — increases quantity if card already exists)
pub fn add_to_collection(
    conn: &rusqlite::Connection,
    card_id: &str,
    quantity: i32,
    condition: &str,
    language: &str,
    is_foil: bool,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO collection (card_id, quantity, condition, language, is_foil)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(card_id) DO UPDATE SET
            quantity = quantity + ?2,
            condition = ?3,
            language = ?4,
            is_foil = ?5",
        rusqlite::params![card_id, quantity, condition, language, is_foil as i32],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get a single collection item by its (auto-)id
pub fn get_collection_item(conn: &rusqlite::Connection, id: i64) -> Result<Option<CollectionItemDb>> {
    let mut stmt = conn.prepare(
        "SELECT id, card_id, quantity, condition, notes, added_at, language, is_foil, acquired_at
         FROM collection WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(rusqlite::params![id], |row| {
        CollectionItemDb::from_row(row)
    })?;

    match rows.next() {
        Some(Ok(item)) => Ok(Some(item)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// Get all collection items, ordered by most recently added
pub fn get_all_collection(conn: &rusqlite::Connection) -> Result<Vec<CollectionItemDb>> {
    let mut stmt = conn.prepare(
        "SELECT id, card_id, quantity, condition, notes, added_at, language, is_foil, acquired_at
         FROM collection ORDER BY added_at DESC",
    )?;

    let rows = stmt.query_map([], |row| CollectionItemDb::from_row(row))?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Get paginated collection items, ordered by most recently added
pub fn get_collection(
    conn: &rusqlite::Connection,
    page: u64,
    per_page: u64,
) -> Result<Vec<CollectionItemDb>> {
    let offset = page.saturating_sub(1) * per_page;
    let mut stmt = conn.prepare(
        "SELECT id, card_id, quantity, condition, notes, added_at, language, is_foil, acquired_at
         FROM collection ORDER BY added_at DESC
         LIMIT ?1 OFFSET ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![per_page, offset], |row| {
        CollectionItemDb::from_row(row)
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Get total number of collection entries
pub fn get_collection_count(conn: &rusqlite::Connection) -> Result<u64> {
    let count: u64 = conn.query_row("SELECT COUNT(*) FROM collection", [], |row| row.get(0))?;
    Ok(count)
}

/// Search collection items by card name (LIKE match)
pub fn search_collection(
    conn: &rusqlite::Connection,
    query: &str,
) -> Result<Vec<CollectionItemDb>> {
    let like_pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT c.id, c.card_id, c.quantity, c.condition, c.notes, c.added_at,
                c.language, c.is_foil, c.acquired_at
         FROM collection c
         JOIN cards card ON c.card_id = card.id
         WHERE card.name LIKE ?1
         ORDER BY c.added_at DESC",
    )?;

    let rows = stmt.query_map(rusqlite::params![like_pattern], |row| {
        CollectionItemDb::from_row(row)
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Update quantity, condition, and notes of a collection item
pub fn update_collection_item(
    conn: &rusqlite::Connection,
    id: i64,
    quantity: i32,
    condition: &str,
    notes: Option<&str>,
) -> Result<usize> {
    conn.execute(
        "UPDATE collection SET quantity = ?1, condition = ?2, notes = ?3 WHERE id = ?4",
        rusqlite::params![quantity, condition, notes, id],
    )
}

/// Remove a card from the collection (by collection id)
pub fn remove_from_collection(conn: &rusqlite::Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM collection WHERE id = ?1", rusqlite::params![id])
}

/// Count total unique cards in the collection
pub fn count_collection(conn: &rusqlite::Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM collection", [], |row| row.get(0))
}

/// Batch import collection items.
/// Resolves card names to Scryfall IDs via the cards table.
pub fn import_collection_batch(
    conn: &rusqlite::Connection,
    items: &[CollectionImportItem],
) -> Result<ImportStats> {
    let mut stats = ImportStats {
        imported: 0,
        updated: 0,
        failed: 0,
        errors: Vec::new(),
    };

    // Prepare lookup: find card_id by name (case-insensitive)
    let mut name_lookup = conn.prepare_cached(
        "SELECT id FROM cards WHERE name = ?1 COLLATE NOCASE LIMIT 1",
    )?;

    for item in items {
        // Try to find card by identifier (could be Scryfall ID or name)
        let card_id: Option<String> = if item.card_identifier.contains('-')
            && item.card_identifier.len() >= 32
        {
            // Looks like a UUID — try direct ID lookup
            conn.query_row(
                "SELECT id FROM cards WHERE id = ?1",
                rusqlite::params![&item.card_identifier],
                |row| row.get(0),
            )
            .ok()
        } else {
            // Try name lookup
            name_lookup
                .query_map(rusqlite::params![&item.card_identifier], |row| {
                    row.get::<_, String>(0)
                })
                .ok()
                .and_then(|mut rows| rows.next().and_then(|r| r.ok()))
        };

        let card_id = match card_id {
            Some(id) => id,
            None => {
                stats.failed += 1;
                stats
                    .errors
                    .push(format!("Card not found: '{}'", item.card_identifier));
                continue;
            }
        };

        // Upsert with the new fields
        let result = conn.execute(
            "INSERT INTO collection (card_id, quantity, condition, language, is_foil)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(card_id) DO UPDATE SET
                 quantity = quantity + ?2,
                 condition = ?3,
                 language = ?4,
                 is_foil = ?5",
            rusqlite::params![
                card_id,
                item.quantity,
                item.condition,
                item.language,
                item.is_foil as i32,
            ],
        );

        match result {
            Ok(affected) => {
                if affected > 0 {
                    stats.imported += 1;
                } else {
                    stats.updated += 1;
                }
            }
            Err(e) => {
                stats.failed += 1;
                stats
                    .errors
                    .push(format!("DB error for '{}': {}", item.card_identifier, e));
            }
        }
    }

    Ok(stats)
}
