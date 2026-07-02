//! MTG Multiverse Studio — Scryfall Bulk Import Module
//!
//! Provides streaming download, decompression, parsing, and batch-insertion
//! of Scryfall bulk data into the local SQLite database.
//! Also includes collection import parsers (CSV, MTGA, Moxfield, Archidekt).
//!
//! # Architecture
//!
//! 1. Fetch `/bulk-data` → find dataset entry → get `download_uri`
//! 2. Download the GZip file (~22 MB for oracle_cards)
//! 3. Decompress
//! 4. Parse and batch-insert into SQLite
//! 5. Rebuild FTS5 index after insert

pub mod collection_import;

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

// ─── Scryfall Bulk Data API Types ─────────────────────────────────

#[derive(Deserialize, Debug)]
struct BulkDataResponse {
    data: Vec<BulkDataEntry>,
}

#[derive(Deserialize, Debug)]
struct BulkDataEntry {
    #[serde(rename = "type")]
    bulk_type: String,
    download_uri: String,
    updated_at: String,
    size: usize,
}

// ─── Dataset Selection ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dataset {
    OracleCards,
    DefaultCards,
    Rulings,
    Sets,
}

/// Progress reporter trait (self-contained, no scryfall dependency)
pub trait ProgressReporter {
    fn report(&self, message: &str);
}

/// Console-based progress reporter
pub struct ConsoleProgress;

impl ProgressReporter for ConsoleProgress {
    fn report(&self, message: &str) {
        tracing::info!("{}", message);
    }
}

// ─── Sets import helpers ──────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct SetsResponse {
    data: Vec<ScryfallSet>,
}

#[derive(Deserialize, Debug)]
struct ScryfallSet {
    id: String,
    code: String,
    name: String,
    set_type: String,
    released_at: Option<String>,
    card_count: u32,
    icon_svg_uri: Option<String>,
    scryfall_uri: Option<String>,
}

// ─── Card Schema (subset of Scryfall oracle_cards) ─────────────────

#[derive(Deserialize, Debug, Clone)]
pub struct ScryfallCardImport {
    pub id: String,
    pub oracle_id: Option<String>,
    pub name: String,
    pub mana_cost: Option<String>,
    pub cmc: Option<f64>,
    #[serde(rename = "type_line")]
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub colors: Option<Vec<String>>,
    pub color_identity: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub legalities: Option<serde_json::Value>,
    pub image_uris: Option<serde_json::Value>,
    pub prices: Option<serde_json::Value>,
    pub released_at: Option<String>,
    pub set_id: Option<String>,
    pub set_name: Option<String>,
    pub set: Option<String>,
    pub collector_number: Option<String>,
    pub rarity: Option<String>,
    pub flavor_text: Option<String>,
    pub artist: Option<String>,
    pub layout: Option<String>,
}

// ─── Download & Import Logic ───────────────────────────────────────

const BULK_DATA_URL: &str = "https://api.scryfall.com/bulk-data";

/// Connect to SQLite with WAL and performance pragmas
fn connect_sqlite(path: &str) -> Result<rusqlite::Connection, Box<dyn std::error::Error>> {
    let conn = rusqlite::Connection::open(path)?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA cache_size=-80000;
         PRAGMA busy_timeout=5000;
         PRAGMA foreign_keys=OFF;",
    )?;

    Ok(conn)
}

/// Create/ensure the cards table exists with full column set
fn ensure_cards_table(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS cards (
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
        )",
    )?;

    // Add columns that may be missing on older DBs (idempotent)
    for col in &[
        "ADD COLUMN oracle_id TEXT",
        "ADD COLUMN power TEXT",
        "ADD COLUMN toughness TEXT",
        "ADD COLUMN released_at TEXT",
        "ADD COLUMN set_name TEXT",
        "ADD COLUMN set_code TEXT",
        "ADD COLUMN collector_number TEXT",
        "ADD COLUMN flavor_text TEXT",
        "ADD COLUMN layout TEXT",
    ] {
        let _ = conn.execute_batch(&format!("ALTER TABLE cards {}", col));
    }

    Ok(())
}

/// Ensure FTS5 virtual table exists and is populated
fn ensure_fts5_index(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS cards_fts USING fts5(
            name, type_line, oracle_text, flavor_text,
            content='cards',
            content_rowid='rowid',
            tokenize='porter unicode61'
        )",
    )?;

    // Populate if empty
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cards_fts",
        [],
        |row| row.get(0),
    )?;

    if count == 0 {
        conn.execute_batch(
            "INSERT OR REPLACE INTO cards_fts(rowid, name, type_line, oracle_text, flavor_text)
             SELECT rowid, name, type_line, oracle_text, flavor_text FROM cards",
        )?;
        info!("FTS5 index populated");
    }

    Ok(())
}

/// Ensure collection table exists with extended columns
fn ensure_collection_table(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS collection (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            condition TEXT NOT NULL DEFAULT 'nm',
            notes TEXT,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            language TEXT NOT NULL DEFAULT 'en',
            is_foil INTEGER NOT NULL DEFAULT 0,
            acquired_at TEXT
        )",
    )?;

    // Add extended columns (idempotent)
    for col in &[
        "ADD COLUMN language TEXT NOT NULL DEFAULT 'en'",
        "ADD COLUMN is_foil INTEGER NOT NULL DEFAULT 0",
        "ADD COLUMN acquired_at TEXT",
    ] {
        let _ = conn.execute_batch(&format!("ALTER TABLE collection {}", col));
    }

    Ok(())
}

// ─── Bulk Import ───────────────────────────────────────────────────

/// Find the download URI for a given bulk dataset type
async fn find_download_uri(
    client: &Client,
    bulk_type: &str,
) -> Result<BulkDataEntry, Box<dyn std::error::Error>> {
    let bulk: BulkDataResponse = client
        .get(BULK_DATA_URL)
        .send()
        .await?
        .json()
        .await?;

    bulk.data
        .into_iter()
        .find(|e| e.bulk_type == bulk_type)
        .ok_or_else(|| format!("No '{}' entry found in bulk data", bulk_type).into())
}

/// Download and parse a JSON array/JSONL response (reqwest handles gzip automatically)
async fn download_json_array(
    client: &Client,
    url: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;
    let total_bytes = response.content_length().unwrap_or(0);

    // reqwest auto-decompresses gzip — read as raw text
    let text = response.text().await?;

    info!("Downloaded {} bytes / {} KB text", total_bytes, text.len() / 1024);

    // The default-cards is a JSON array, oracle_cards is JSONL.
    // We handle both by checking the first char.
    if text.trim_start().starts_with('[') {
        // JSON array: parse as Vec<Value>
        let arr: Vec<serde_json::Value> = serde_json::from_str(&text)?;
        Ok(arr)
    } else {
        // JSONL: each line is one JSON object
        let mut arr = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    arr.push(val);
                }
            }
        }
        Ok(arr)
    }
}

/// Download and bulk-insert all cards
async fn run_cards_import(
    conn: &rusqlite::Connection,
    client: &Client,
    bulk_type: &str,
    reporter: &ConsoleProgress,
) -> Result<usize, Box<dyn std::error::Error>> {
    let entry = find_download_uri(client, bulk_type).await?;
    reporter.report(&format!(
        "{}: {} bytes, updated {}",
        bulk_type, entry.size, entry.updated_at
    ));

    let cards_json = download_json_array(client, &entry.download_uri).await?;
    let total = cards_json.len();
    reporter.report(&format!("Parsed {} cards from JSON", total));

    // Batch insert
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO cards (
            id, oracle_id, name, mana_cost, cmc, type_line, oracle_text,
            power, toughness, colors, color_identity, keywords,
            legalities, image_uris_json, prices, released_at,
            set_id, set_name, set_code, collector_number, rarity,
            flavor_text, artist, layout
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
          ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)"
    )?;

    conn.execute_batch("BEGIN")?;
    let mut count = 0;
    let batch_size = 100;

    for card_val in &cards_json {
        let id = card_val["id"].as_str().unwrap_or("").to_string();
        let name = card_val["name"].as_str().unwrap_or("Unknown").to_string();

        // Skip double-faced / split cards that don't have their own ID
        if id.is_empty() {
            warn!("Skipping card '{}' without id", name);
            continue;
        }

        let colors = card_val.get("colors")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "[]".to_string());
        let color_identity = card_val.get("color_identity")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "[]".to_string());
        let keywords = card_val.get("keywords")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "[]".to_string());
        let legalities = card_val.get("legalities")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_string());
        let image_uris = card_val.get("image_uris")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_string());
        let prices = card_val.get("prices")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_string());

        stmt.execute(rusqlite::params![
            id,
            card_val["oracle_id"].as_str(),
            name,
            card_val["mana_cost"].as_str(),
            card_val["cmc"].as_f64(),
            card_val["type_line"].as_str(),
            card_val["oracle_text"].as_str(),
            card_val["power"].as_str(),
            card_val["toughness"].as_str(),
            colors,
            color_identity,
            keywords,
            legalities,
            image_uris,
            prices,
            card_val["released_at"].as_str(),
            card_val["set_id"].as_str(),
            card_val["set_name"].as_str(),
            card_val["set"].as_str(),
            card_val["collector_number"].as_str(),
            card_val["rarity"].as_str(),
            card_val["flavor_text"].as_str(),
            card_val["artist"].as_str(),
            card_val["layout"].as_str(),
        ])?;

        count += 1;

        if count % batch_size == 0 {
            conn.execute_batch("COMMIT")?;
            conn.execute_batch("BEGIN")?;
            if count % 5000 == 0 {
                reporter.report(&format!("Imported {} / {} cards…", count, total));
            }
        }
    }

    conn.execute_batch("COMMIT")?;
    reporter.report(&format!("Inserted {} / {} cards", count, total));

    // Rebuild FTS5 index
    ensure_fts5_index(conn)?;

    Ok(count)
}

/// Download and import all sets from Scryfall
pub async fn run_sets_import(
    conn: &rusqlite::Connection,
    client: &Client,
    reporter: &ConsoleProgress,
) -> Result<usize, Box<dyn std::error::Error>> {
    reporter.report("Fetching sets from Scryfall…");

    let resp: SetsResponse = client
        .get("https://api.scryfall.com/sets")
        .send()
        .await?
        .json()
        .await?;

    let total = resp.data.len();
    reporter.report(&format!("Found {} sets", total));

    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO sets (id, code, name, set_type, released_at, card_count, icon_svg_uri, scryfall_uri)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?;

    conn.execute_batch("BEGIN")?;
    let mut count = 0;

    for set in &resp.data {
        stmt.execute(rusqlite::params![
            set.code,  // id
            set.code,  // code
            set.name,
            set.set_type,
            set.released_at,
            set.card_count,
            set.icon_svg_uri,
            set.scryfall_uri,
        ])?;
        count += 1;
    }

    conn.execute_batch("COMMIT")?;
    reporter.report(&format!("Imported {} sets", count));

    Ok(count)
}

/// Run a bulk import of the given dataset
pub async fn run_bulk_import(
    conn: &rusqlite::Connection,
    client: &Client,
    dataset: Dataset,
    reporter: &ConsoleProgress,
) -> Result<usize, Box<dyn std::error::Error>> {
    match dataset {
        Dataset::OracleCards | Dataset::DefaultCards => {
            let bulk_type = match dataset {
                Dataset::OracleCards => "oracle_cards",
                Dataset::DefaultCards => "default_cards",
                _ => unreachable!(),
            };
            run_cards_import(conn, client, bulk_type, reporter).await
        }
        Dataset::Sets => {
            run_sets_import(conn, client, reporter).await
        }
        Dataset::Rulings => {
            // Rulings import is a separate JSON format; stub for now
            reporter.report("Rulings import not yet implemented");
            Ok(0)
        }
    }
}

// ─── Oracle Cards Entry Point ──────────────────────────────────────

/// Standalone entry point: Download and import Oracle cards
pub async fn import_oracle_cards(db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let client = Client::builder()
        .user_agent("MTG-Multiverse-Studio/0.1")
        .build()?;

    let conn = connect_sqlite(db_path)?;
    ensure_cards_table(&conn)?;
    ensure_collection_table(&conn)?;
    ensure_fts5_index(&conn)?;

    let reporter = ConsoleProgress;

    // Import sets first so card FK references are satisfied
    info!("Step 1/2: Importing sets…");
    if let Err(e) = run_sets_import(&conn, &client, &reporter).await {
        warn!("Sets import issue (non-fatal, cards may lack set names): {}", e);
    }

    info!("Step 2/2: Importing oracle cards…");
    run_bulk_import(&conn, &client, Dataset::OracleCards, &reporter).await?;

    info!("Full import completed in {:?}", start.elapsed());
    Ok(())
}
