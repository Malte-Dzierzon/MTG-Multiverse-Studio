//! Lore Repository
//! 
//! Handles database operations for the `lore_entries` table.

use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

/// Lore entry from SQLite
#[derive(Debug, Clone)]
pub struct LoreDb {
    pub id: i64,
    pub title: String,
    pub lore_type: String,
    pub content_path: Option<String>,
    pub metadata: String,
    pub related_cards: String,
}

/// Insert a new lore entry
pub fn insert_lore_entry(
    conn: &rusqlite::Connection,
    title: &str,
    lore_type: &str,
    content_path: Option<&str>,
    metadata: Option<&str>,
    related_cards: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO lore_entries (title, lore_type, content_path, metadata, related_cards)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            title,
            lore_type,
            content_path,
            metadata.unwrap_or("{}"),
            related_cards.unwrap_or("[]"),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get a single lore entry by ID
pub fn get_lore_entry_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Option<LoreDb>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, lore_type, content_path, metadata, related_cards
         FROM lore_entries WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        Ok(LoreDb {
            id: row.get(0)?,
            title: row.get(1)?,
            lore_type: row.get(2)?,
            content_path: row.get(3)?,
            metadata: row.get(4)?,
            related_cards: row.get(5)?,
        })
    })?;

    match rows.next() {
        Some(Ok(entry)) => Ok(Some(entry)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// Get all lore entries, optionally filtered by type
pub fn get_all_lore_entries(
    conn: &rusqlite::Connection,
    lore_type: Option<&str>,
) -> Result<Vec<LoreDb>> {
    let (query, param): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(lt) = lore_type {
        (
            "SELECT id, title, lore_type, content_path, metadata, related_cards
             FROM lore_entries WHERE lore_type = ?1 ORDER BY id".to_string(),
            vec![Box::new(lt.to_string())],
        )
    } else {
        (
            "SELECT id, title, lore_type, content_path, metadata, related_cards
             FROM lore_entries ORDER BY id".to_string(),
            vec![],
        )
    };

    let mut stmt = conn.prepare(&query)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param.iter().map(|p| p.as_ref()).collect();
    
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(LoreDb {
            id: row.get(0)?,
            title: row.get(1)?,
            lore_type: row.get(2)?,
            content_path: row.get(3)?,
            metadata: row.get(4)?,
            related_cards: row.get(5)?,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

/// Update a lore entry
pub fn update_lore_entry(
    conn: &rusqlite::Connection,
    id: i64,
    title: &str,
    lore_type: &str,
    content_path: Option<&str>,
    metadata: Option<&str>,
    related_cards: Option<&str>,
) -> Result<usize> {
    conn.execute(
        "UPDATE lore_entries SET title = ?1, lore_type = ?2, content_path = ?3, metadata = ?4, related_cards = ?5
         WHERE id = ?6",
        params![
            title,
            lore_type,
            content_path,
            metadata.unwrap_or("{}"),
            related_cards.unwrap_or("[]"),
            id,
        ],
    )
}

/// Delete a lore entry
pub fn delete_lore_entry(conn: &rusqlite::Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM lore_entries WHERE id = ?1", params![id])
}
