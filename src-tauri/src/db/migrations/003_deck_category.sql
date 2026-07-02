-- Migration: Deck Cards Category
-- Created: 2026-07-01
-- Description: Adds category column to deck_cards for mainboard/sideboard/maybe support

-- Check if column exists before adding
-- SQLite doesn't support ADD COLUMN IF NOT EXISTS, so we use a trick
-- This migration should only run once, but we make it idempotent by checking first
-- We'll use a migration tracking table instead
