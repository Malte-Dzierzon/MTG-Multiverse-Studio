//! Lore Repository
//! 
//! Handles database operations for the `lore_entries` table.

use rusqlite::{params, Result};

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

/// Search lore entries by title (LIKE query)
pub fn search_lore_entries(
    conn: &rusqlite::Connection,
    query: &str,
) -> Result<Vec<LoreDb>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, title, lore_type, content_path, metadata, related_cards
         FROM lore_entries WHERE title LIKE ?1 ORDER BY id",
    )?;

    let rows = stmt.query_map(params![pattern], |row| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::init_test_db;

    #[test]
    fn test_search_lore_empty_db() {
        let conn = init_test_db().expect("Failed to create test database");
        let results = search_lore_entries(&conn, "nonexistent").unwrap();
        assert!(results.is_empty(), "Expected empty results, got {}", results.len());
    }

    #[test]
    fn test_search_lore_with_results() {
        let conn = init_test_db().expect("Failed to create test database");

        // Insert a few entries
        insert_lore_entry(
            &conn,
            "War of the Spark",
            "saga",
            Some("stories/war-of-the-spark.md"),
            None,
            None,
        )
        .unwrap();

        insert_lore_entry(
            &conn,
            "The Brothers' War",
            "saga",
            Some("stories/brothers-war.md"),
            None,
            None,
        )
        .unwrap();

        insert_lore_entry(
            &conn,
            "Gideon's Origin",
            "character",
            Some("stories/gideon.md"),
            None,
            None,
        )
        .unwrap();

        // Search for "war" — should match 2 entries
        let results = search_lore_entries(&conn, "war").unwrap();
        assert_eq!(results.len(), 2, "Expected 2 entries for 'war'");
        assert!(results.iter().any(|r| r.title == "War of the Spark"));
        assert!(results.iter().any(|r| r.title == "The Brothers' War"));

        // Search for "Brothers" — should match 1
        let results = search_lore_entries(&conn, "Brothers").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "The Brothers' War");

        // Search for nonexistent
        let results = search_lore_entries(&conn, "zzzzzzz").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_lore_case_sensitivity() {
        let conn = init_test_db().expect("Failed to create test database");

        insert_lore_entry(
            &conn,
            "Dominaria",
            "plane",
            Some("stories/dominaria.md"),
            None,
            None,
        )
        .unwrap();

        // SQLite LIKE is case-insensitive for ASCII by default
        let results = search_lore_entries(&conn, "dominaria").unwrap();
        assert_eq!(results.len(), 1, "Expected 1 entry for lowercase 'dominaria'");
        assert_eq!(results[0].title, "Dominaria");

        let results = search_lore_entries(&conn, "DOM").unwrap();
        assert_eq!(results.len(), 1, "Expected 1 entry for uppercase 'DOM'");
    }
}
