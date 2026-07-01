//! Database Connection Management
//! 
//! Handles SQLite connection initialization, migrations, and connection management.

use rusqlite::{Connection, Result};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

/// Get the database file path in the app's data directory
pub fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    // Create app data directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    
    Ok(app_data_dir.join("mtg_multiverse_studio.db"))
}

/// Initialize the database connection and run migrations
pub fn init_db(app_handle: &AppHandle) -> Result<Connection> {
    let db_path = get_db_path(app_handle)?;
    
    // Open connection (creates file if it doesn't exist)
    let conn = Connection::open(&db_path)?;
    
    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "foreign_keys", &"ON")?;
    conn.pragma_update(None, "busy_timeout", &5000)?; // 5 second busy timeout
    
    // Run migrations
    run_migrations(&conn)?;
    
    tracing::info!("Database initialized at: {:?}", db_path);
    Ok(conn)
}

/// Run all pending migrations (all .sql files in migrations/ dir, alphabetical order)
fn run_migrations(conn: &Connection) -> Result<()> {
    let migrations: &[(&str, &str)] = &[
        ("001_initial", include_str!("migrations/001_initial.sql")),
        ("002_fts5", include_str!("migrations/002_fts5.sql")),
        ("003_deck_category", include_str!("migrations/003_deck_category.sql")),
        ("004_collection_ext", include_str!("migrations/004_collection_ext.sql")),
    ];

    for (name, sql) in migrations {
        conn.execute_batch(sql)?;
        tracing::info!("Migration '{}' applied successfully", name);
    }

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

/// Initialize an in-memory database for testing
#[cfg(test)]
pub fn init_test_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "foreign_keys", &"ON")?;
    
    run_migrations(&conn)?;
    
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_initialization() {
        let conn = init_test_db().expect("Failed to create test database");
        
        // Verify tables exist by querying sqlite_master
        let table_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        ).expect("Failed to query table count");
        
        // Should have at least 6 tables: sets, cards, collection, decks, deck_cards, lore_entries
        assert!(table_count >= 6, "Expected at least 6 tables, got {}", table_count);
        
        // Verify indexes exist
        let index_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
            [],
            |row| row.get(0),
        ).expect("Failed to query index count");
        
        assert!(index_count > 0, "Expected indexes to be created");
    }
    
    #[test]
    fn test_cards_table_structure() {
        let conn = init_test_db().expect("Failed to create test database");
        
        // Check columns in cards table
        let cols: Vec<String> = conn.prepare("PRAGMA table_info(cards)")
            .expect("Failed to prepare pragma")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("Failed to query columns")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect columns");
        
        assert!(cols.contains(&"id".to_string()));
        assert!(cols.contains(&"name".to_string()));
        assert!(cols.contains(&"oracle_id".to_string()));
        assert!(cols.contains(&"mana_cost".to_string()));
        assert!(cols.contains(&"cmc".to_string()));
        assert!(cols.contains(&"type_line".to_string()));
        assert!(cols.contains(&"oracle_text".to_string()));
        assert!(cols.contains(&"colors".to_string()));
        assert!(cols.contains(&"color_identity".to_string()));
        assert!(cols.contains(&"keywords".to_string()));
        assert!(cols.contains(&"rarity".to_string()));
        assert!(cols.contains(&"set_id".to_string()));
        assert!(cols.contains(&"image_uris".to_string()));
        assert!(cols.contains(&"artist".to_string()));
        assert!(cols.contains(&"legalities".to_string()));
        assert!(cols.contains(&"prices".to_string()));
    }
}