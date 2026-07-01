-- Migration: Deck Cards Category
-- Created: 2026-07-01
-- Description: Adds category column to deck_cards for mainboard/sideboard/maybe support

ALTER TABLE deck_cards ADD COLUMN category TEXT DEFAULT 'mainboard';
