-- Migration: Price History Table
-- Created: 2026-07-01
-- Description: Creates a historical price tracking table for MTG cards

-- ============================================
-- PRICE HISTORY TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id TEXT NOT NULL REFERENCES cards(id),
    source TEXT NOT NULL,  -- 'scryfall', 'cardmarket', 'tcgplayer'
    currency TEXT DEFAULT 'EUR',
    price_low REAL,
    price_avg REAL,
    price_high REAL,
    price_trend REAL,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_price_history_card ON price_history(card_id, source);
CREATE INDEX IF NOT EXISTS idx_price_history_fetched ON price_history(fetched_at);
