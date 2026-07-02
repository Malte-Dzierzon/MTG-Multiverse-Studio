-- Migration: Initial Schema
-- Created: 2026-07-01
-- Description: Creates all tables for MTG Multiverse Studio

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ============================================
-- SETS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS sets (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    set_type TEXT,
    released_at DATE,
    collector_number_prefix TEXT,
    card_count INTEGER,
    icon_svg_uri TEXT,
    scryfall_uri TEXT
);

-- ============================================
-- CARDS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    oracle_id TEXT,
    name TEXT NOT NULL,
    mana_cost TEXT,
    cmc REAL,
    type_line TEXT,
    oracle_text TEXT,
    power TEXT,
    toughness TEXT,
    colors TEXT DEFAULT '[]',
    color_identity TEXT DEFAULT '[]',
    keywords TEXT DEFAULT '[]',
    legalities TEXT DEFAULT '{}',
    image_uris_json TEXT DEFAULT '{}',
    prices TEXT DEFAULT '{}',
    released_at TEXT,
    set_id TEXT,
    set_name TEXT,
    set_code TEXT,
    collector_number TEXT,
    rarity TEXT,
    flavor_text TEXT,
    artist TEXT,
    layout TEXT
);

CREATE INDEX IF NOT EXISTS idx_cards_name ON cards(name);
CREATE INDEX IF NOT EXISTS idx_cards_oracle_id ON cards(oracle_id);
CREATE INDEX IF NOT EXISTS idx_cards_set_rarity ON cards(set_id, rarity);
CREATE INDEX IF NOT EXISTS idx_cards_cmc ON cards(cmc);

-- ============================================
-- COLLECTION TABLE (User's card collection)
-- ============================================
CREATE TABLE IF NOT EXISTS collection (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id TEXT NOT NULL UNIQUE REFERENCES cards(id),
    quantity INTEGER DEFAULT 1,
    condition TEXT DEFAULT 'nm',
    notes TEXT,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_collection_card ON collection(card_id);

-- ============================================
-- DECKS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS decks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    format TEXT,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);

-- ============================================
-- DECK_CARDS TABLE (Many-to-Many: Decks <-> Cards)
-- ============================================
CREATE TABLE IF NOT EXISTS deck_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    card_id TEXT NOT NULL REFERENCES cards(id),
    quantity INTEGER DEFAULT 1,
    position INTEGER,
    category TEXT DEFAULT 'mainboard',
    UNIQUE(deck_id, card_id)
);

CREATE INDEX IF NOT EXISTS idx_deck_cards_deck ON deck_cards(deck_id);
CREATE INDEX IF NOT EXISTS idx_deck_cards_card ON deck_cards(card_id);
CREATE INDEX IF NOT EXISTS idx_deck_cards_lookup ON deck_cards(deck_id, card_id);

-- ============================================
-- LORE_ENTRIES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS lore_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    lore_type TEXT,
    content_path TEXT,
    metadata TEXT DEFAULT '{}',
    related_cards TEXT DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_lore_type ON lore_entries(lore_type);
CREATE INDEX IF NOT EXISTS idx_lore_title ON lore_entries(title);

-- ============================================
-- OPTIONAL: FTS5 Full-Text Search for Cards
-- Uncomment to enable (requires fts5 feature in rusqlite)
-- ============================================
-- CREATE VIRTUAL TABLE IF NOT EXISTS cards_fts USING fts5(
--     name,
--     type_line,
--     oracle_text,
--     content='cards',
--     content_rowid='rowid'
-- );
--
-- CREATE TRIGGER IF NOT EXISTS cards_ai AFTER INSERT ON cards BEGIN
--     INSERT INTO cards_fts(rowid, name, type_line, oracle_text)
--     VALUES (new.rowid, new.name, new.type_line, new.oracle_text);
-- END;
--
-- CREATE TRIGGER IF NOT EXISTS cards_ad AFTER DELETE ON cards BEGIN
--     INSERT INTO cards_fts(cards_fts, rowid, name, type_line, oracle_text)
--     VALUES ('delete', old.rowid, old.name, old.type_line, old.oracle_text);
-- END;
--
-- CREATE TRIGGER IF NOT EXISTS cards_au AFTER UPDATE ON cards BEGIN
--     INSERT INTO cards_fts(cards_fts, rowid, name, type_line, oracle_text)
--     VALUES ('delete', old.rowid, old.name, old.type_line, old.oracle_text);
--     INSERT INTO cards_fts(rowid, name, type_line, oracle_text)
--     VALUES (new.rowid, new.name, new.type_line, new.oracle_text);
-- END;