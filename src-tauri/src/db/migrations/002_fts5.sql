-- Migration: FTS5 Full-Text Search for Cards
-- Created: 2026-07-01
-- Description: Enables fast full-text search on card name, type_line, oracle_text, and flavor_text

-- ============================================
-- FTS5 VIRTUAL TABLE
-- ============================================
CREATE VIRTUAL TABLE IF NOT EXISTS cards_fts USING fts5(
    name,
    type_line,
    oracle_text,
    flavor_text,
    content='cards',
    content_rowid='rowid',
    tokenize='unicode61'
);

-- ============================================
-- TRIGGERS: Keep FTS index in sync with cards table
-- ============================================

-- After INSERT: add new card to FTS index
CREATE TRIGGER IF NOT EXISTS cards_ai AFTER INSERT ON cards BEGIN
    INSERT INTO cards_fts(rowid, name, type_line, oracle_text, flavor_text)
    VALUES (new.rowid, new.name, new.type_line, new.oracle_text, new.flavor_text);
END;

-- After DELETE: remove card from FTS index
CREATE TRIGGER IF NOT EXISTS cards_ad AFTER DELETE ON cards BEGIN
    INSERT INTO cards_fts(cards_fts, rowid, name, type_line, oracle_text, flavor_text)
    VALUES ('delete', old.rowid, old.name, old.type_line, old.oracle_text, old.flavor_text);
END;

-- After UPDATE: replace card in FTS index (delete old, insert new)
CREATE TRIGGER IF NOT EXISTS cards_au AFTER UPDATE ON cards BEGIN
    INSERT INTO cards_fts(cards_fts, rowid, name, type_line, oracle_text, flavor_text)
    VALUES ('delete', old.rowid, old.name, old.type_line, old.oracle_text, old.flavor_text);
    INSERT INTO cards_fts(rowid, name, type_line, oracle_text, flavor_text)
    VALUES (new.rowid, new.name, new.type_line, new.oracle_text, new.flavor_text);
END;

-- ============================================
-- POPULATE FTS INDEX WITH EXISTING DATA
-- ============================================
INSERT INTO cards_fts(rowid, name, type_line, oracle_text, flavor_text)
SELECT rowid, name, type_line, oracle_text, flavor_text FROM cards;
