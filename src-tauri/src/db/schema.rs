//! Database Schema Definitions
//! 
//! Contains all CREATE TABLE statements and index definitions for the MTG Multiverse Studio database.

/// SQL for creating the `sets` table
pub const CREATE_SETS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS sets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    set_type TEXT,
    released_at DATE,
    collector_number_prefix TEXT
);
"#;

/// SQL for creating the `cards` table
pub const CREATE_CARDS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    oracle_id TEXT UNIQUE,
    name TEXT NOT NULL,
    mana_cost TEXT,
    cmc REAL,
    type_line TEXT,
    oracle_text TEXT,
    colors TEXT DEFAULT '[]',
    color_identity TEXT DEFAULT '[]',
    keywords TEXT DEFAULT '[]',
    rarity TEXT,
    set_id TEXT REFERENCES sets(id),
    image_uris TEXT,
    artist TEXT,
    legalities TEXT DEFAULT '{}',
    prices TEXT DEFAULT '{}'
);
"#;

/// Indexes for the `cards` table
pub const CREATE_CARDS_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_cards_name ON cards(name);
CREATE INDEX IF NOT EXISTS idx_cards_oracle_id ON cards(oracle_id);
CREATE INDEX IF NOT EXISTS idx_cards_set_rarity ON cards(set_id, rarity);
CREATE INDEX IF NOT EXISTS idx_cards_cmc ON cards(cmc);
"#;

/// SQL for creating the `collection` table (user's card collection)
pub const CREATE_COLLECTION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS collection (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id TEXT NOT NULL UNIQUE REFERENCES cards(id),
    quantity INTEGER DEFAULT 1,
    condition TEXT DEFAULT 'nm',
    notes TEXT,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
"#;

/// Index for the `collection` table
pub const CREATE_COLLECTION_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_collection_card ON collection(card_id);
"#;

/// SQL for creating the `decks` table
pub const CREATE_DECKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS decks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    format TEXT,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);
"#;

/// SQL for creating the `deck_cards` table (many-to-many: decks <-> cards)
pub const CREATE_DECK_CARDS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS deck_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    card_id TEXT NOT NULL REFERENCES cards(id),
    quantity INTEGER DEFAULT 1,
    position INTEGER,
    UNIQUE(deck_id, card_id)
);
"#;

/// Indexes for the `deck_cards` table
pub const CREATE_DECK_CARDS_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_deck_cards_deck ON deck_cards(deck_id);
CREATE INDEX IF NOT EXISTS idx_deck_cards_card ON deck_cards(card_id);
CREATE INDEX IF NOT EXISTS idx_deck_cards_lookup ON deck_cards(deck_id, card_id);
"#;

/// SQL for creating the `lore_entries` table
pub const CREATE_LORE_ENTRIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS lore_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    lore_type TEXT,
    content_path TEXT,
    metadata TEXT DEFAULT '{}',
    related_cards TEXT DEFAULT '[]'
);
"#;

/// Indexes for the `lore_entries` table
pub const CREATE_LORE_ENTRIES_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_lore_type ON lore_entries(lore_type);
CREATE INDEX IF NOT EXISTS idx_lore_title ON lore_entries(title);
"#;

/// FTS5 Virtual Table for full-text search on cards (optional, can be enabled later)
pub const CREATE_CARDS_FTS: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS cards_fts USING fts5(
    name,
    type_line,
    oracle_text,
    content='cards',
    content_rowid='rowid'
);
"#;

/// Triggers to keep FTS table in sync with cards table
pub const CREATE_CARDS_FTS_TRIGGERS: &str = r#"
CREATE TRIGGER IF NOT EXISTS cards_ai AFTER INSERT ON cards BEGIN
    INSERT INTO cards_fts(rowid, name, type_line, oracle_text)
    VALUES (new.rowid, new.name, new.type_line, new.oracle_text);
END;

CREATE TRIGGER IF NOT EXISTS cards_ad AFTER DELETE ON cards BEGIN
    INSERT INTO cards_fts(cards_fts, rowid, name, type_line, oracle_text)
    VALUES ('delete', old.rowid, old.name, old.type_line, old.oracle_text);
END;

CREATE TRIGGER IF NOT EXISTS cards_au AFTER UPDATE ON cards BEGIN
    INSERT INTO cards_fts(cards_fts, rowid, name, type_line, oracle_text)
    VALUES ('delete', old.rowid, old.name, old.type_line, old.oracle_text);
    INSERT INTO cards_fts(rowid, name, type_line, oracle_text)
    VALUES (new.rowid, new.name, new.type_line, new.oracle_text);
END;
"#;

/// All schema statements in order of execution
pub const ALL_SCHEMA_STATEMENTS: &[&str] = &[
    CREATE_SETS_TABLE,
    CREATE_CARDS_TABLE,
    CREATE_CARDS_INDEXES,
    CREATE_COLLECTION_TABLE,
    CREATE_COLLECTION_INDEXES,
    CREATE_DECKS_TABLE,
    CREATE_DECK_CARDS_TABLE,
    CREATE_DECK_CARDS_INDEXES,
    CREATE_LORE_ENTRIES_TABLE,
    CREATE_LORE_ENTRIES_INDEXES,
    // FTS5 is optional - enable by uncommenting:
    // CREATE_CARDS_FTS,
    // CREATE_CARDS_FTS_TRIGGERS,
];

/// Initialize database schema - run all CREATE statements
pub fn init_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "foreign_keys", &"ON")?;
    
    for stmt in ALL_SCHEMA_STATEMENTS {
        conn.execute_batch(stmt)?;
    }
    
    Ok(())
}