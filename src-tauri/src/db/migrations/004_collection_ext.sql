-- Migration: Collection Schema Extension
-- Created: 2026-07-01
-- Description: Adds language, foil, and acquired_at columns to collection table
--              for better collection management.

-- Add new columns to collection table
ALTER TABLE collection ADD COLUMN language TEXT DEFAULT 'en';
ALTER TABLE collection ADD COLUMN is_foil INTEGER DEFAULT 0;
ALTER TABLE collection ADD COLUMN acquired_at DATE;
