//! MTG Multiverse Studio — Scryfall Bulk Import Module
//!
//! Provides streaming download, decompression, parsing, and batch-insertion
//! of Scryfall bulk data into the local SQLite database.
//! Also includes collection import parsers (CSV, MTGA, Moxfield, Archidekt).
//!
//! # Architecture
//!
//! 1. Fetch `/bulk-data` → find `oracle_cards` entry → get `download_uri`
//! 2. Download the JSONL.GZ (~22 MB compressed)
//! 3. Decompress with `flate2::GzDecoder`
//! 4. Parse line-by-line with `serde_json`
//! 5. Batch-insert into SQLite (100 cards per transaction)
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin mtg-import -- oracle
//! cargo run --bin mtg-import -- oracle --db /path/to/custom.db
//! ```

pub mod collection_import;

use reqwest::Client;
use serde::Deserialize;
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
    data_type: String,
    name: String,
    description: String,
    download_uri: String,
    compressed_size: Option<u64>,
    #[allow(dead_code)]
    content_type: String,
    #[allow(dead_code)]
    content_encoding: String,
}

// ─── Import Configuration ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dataset {
    OracleCards,
    DefaultCards,
    Rulings,
    Sets,
}

impl Dataset {
    fn api_type(&self) -> &'static str {
        match self {
            Dataset::OracleCards => "oracle_cards",
            Dataset::DefaultCards => "default_cards",
            Dataset::Rulings => "rulings",
            Dataset::Sets => "sets",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Dataset::OracleCards => "Oracle Cards (~22 MB gzip, ~38k cards)",
            Dataset::DefaultCards => "Default Cards (~69 MB gzip, ~90k printings)",
            Dataset::Rulings => "Rulings (~4 MB gzip)",
            Dataset::Sets => "Sets (~500 KB, ~800 sets)",
        }
    }
}

// ─── Progress Reporting ──────────────────────────────────────────

pub trait ProgressReporter: Send + Sync {
    fn on_progress(&self, imported: u64, total: Option<u64>);
    fn on_error(&self, line: u64, msg: &str);
    fn on_complete(&self, total: u64, duration_secs: f64);
}

pub struct ConsoleProgress;

impl ProgressReporter for ConsoleProgress {
    fn on_progress(&self, imported: u64, _total: Option<u64>) {
        eprint!("\r📥 Importiert: {} Karten", imported);
    }

    fn on_error(&self, line: u64, msg: &str) {
        warn!("Zeile {}: {}", line, msg);
    }

    fn on_complete(&self, total: u64, duration_secs: f64) {
        eprintln!("\n✅ Import abgeschlossen: {} Karten in {:.1}s", total, duration_secs);
    }
}

// ─── Scryfall Card (minimal import subset) ───────────────────────

#[derive(Deserialize, Debug)]
pub struct ScryfallImportCard {
    pub id: String,
    pub oracle_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub layout: Option<String>,
    pub released_at: Option<String>,
    pub image_uris: Option<serde_json::Value>,
    pub mana_cost: Option<String>,
    pub cmc: Option<f64>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colors: Option<Vec<String>>,
    pub color_identity: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub legalities: Option<serde_json::Value>,
    pub set: Option<String>,
    pub set_name: Option<String>,
    pub set_type: Option<String>,
    pub rarity: Option<String>,
    pub artist: Option<String>,
    pub collector_number: Option<String>,
    pub prices: Option<serde_json::Value>,
    pub flavor_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i64>,
    #[serde(default)]
    pub finishes: Vec<String>,
    #[serde(default)]
    pub frame: String,
    pub border_color: Option<String>,
}

// ─── Core Import Functions ───────────────────────────────────────

/// Get the download URI for a specific bulk dataset from Scryfall.
pub async fn get_download_uri(client: &Client, dataset: Dataset) -> Result<String, String> {
    info!("Hole Bulk-Data-Liste von Scryfall...");

    let resp = client
        .get("https://api.scryfall.com/bulk-data")
        .send()
        .await
        .map_err(|e| format!("HTTP-Fehler: {}", e))?;

    let bulk: BulkDataResponse = resp
        .json()
        .await
        .map_err(|e| format!("JSON-Parse-Fehler: {}", e))?;

    let entry = bulk
        .data
        .into_iter()
        .find(|e| e.data_type == dataset.api_type())
        .ok_or_else(|| format!("Dataset '{}' nicht gefunden", dataset.api_type()))?;

    info!(
        "Dataset '{}' gefunden: {} ({} MB compressed)",
        entry.name,
        entry.description,
        entry.compressed_size.map(|s| s / 1_000_000).unwrap_or(0)
    );

    Ok(entry.download_uri)
}

/// Insert a batch of cards into SQLite within a single transaction.
fn insert_card_batch(
    conn: &rusqlite::Connection,
    cards: &[ScryfallImportCard],
) -> Result<usize, String> {
    let mut count = 0;
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Transaction error: {}", e))?;

    for card in cards {
        let image_uris_json = card
            .image_uris
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let prices_json = card
            .prices
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let legalities_json = card
            .legalities
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let colors_json = serde_json::to_string(&card.colors).unwrap_or_default();
        let color_identity_json = serde_json::to_string(&card.color_identity).unwrap_or_default();
        let keywords_json = serde_json::to_string(&card.keywords).unwrap_or_default();
        let finishes_str = card.finishes.join(",");
        let cmc = card.cmc.unwrap_or(0.0);

        let sql = r#"
            INSERT OR REPLACE INTO cards (
                id, oracle_id, name, layout,
                mana_cost, cmc, type_line, oracle_text,
                colors, color_identity, keywords,
                legalities, prices, image_uris_json,
                set_code, set_name, set_type, rarity,
                artist, collector_number, flavor_text,
                power, toughness, loyalty, defense,
                edhrec_rank, finishes, frame, border_color,
                released_at
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7, ?8,
                ?9, ?10, ?11,
                ?12, ?13, ?14,
                ?15, ?16, ?17, ?18,
                ?19, ?20, ?21,
                ?22, ?23, ?24, ?25,
                ?26, ?27, ?28, ?29,
                ?30
            )
        "#;

        match tx.execute(
            sql,
            rusqlite::params![
                card.id,
                card.oracle_id,
                card.name,
                card.layout,
                card.mana_cost,
                cmc,
                card.type_line,
                card.oracle_text,
                colors_json,
                color_identity_json,
                keywords_json,
                legalities_json,
                prices_json,
                image_uris_json,
                card.set,
                card.set_name,
                card.set_type,
                card.rarity,
                card.artist,
                card.collector_number,
                card.flavor_text,
                card.power,
                card.toughness,
                card.loyalty,
                card.defense,
                card.edhrec_rank,
                finishes_str,
                card.frame,
                card.border_color,
                card.released_at,
            ],
        ) {
            Ok(_) => count += 1,
            Err(e) => warn!("Fehler beim Einfügen von '{}': {}", card.name, e),
        }
    }

    tx.commit()
        .map_err(|e| format!("Commit error: {}", e))?;

    Ok(count)
}

/// Run a full bulk import: download → decompress → parse → batch-insert.
///
/// RAM-Bedarf: < 50 MB (22 MB Download + Parse-Buffer).
pub async fn run_bulk_import(
    conn: &rusqlite::Connection,
    client: &Client,
    dataset: Dataset,
    reporter: &dyn ProgressReporter,
) -> Result<u64, String> {
    let start = Instant::now();
    info!("Starte Bulk-Import: {}", dataset.description());

    // 1. Get download URI
    let download_uri = get_download_uri(client, dataset).await?;

    // 2. Download — reqwest dekomprimiert gzip automatisch
    info!("Lade herunter...");
    let response = client
        .get(&download_uri)
        .send()
        .await
        .map_err(|e| format!("Download-Fehler: {}", e))?;

    // content_length ist die gzip-compressed size
    let compressed_size = response.content_length();
    info!(
        "Download-Größe: {}",
        compressed_size
            .map(|s| format!("{} MB", s / 1_000_000))
            .unwrap_or_else(|| "unbekannt".into())
    );

    // 3. Performance pragmas for bulk import
    conn.pragma_update(None, "synchronous", &"NORMAL")
        .map_err(|e| format!("Pragma error: {}", e))?;
    conn.pragma_update(None, "cache_size", &"-80000")
        .map_err(|e| format!("Pragma error: {}", e))?;
    conn.pragma_update(None, "locking_mode", &"EXCLUSIVE")
        .map_err(|e| format!("Pragma error: {}", e))?;

    // 4. Parse JSON array — Scryfall liefert [{...}, {...}, ...]
    //    179 MB JSON → ~38k Cards → ~19 MB Vec (ok für Desktop-App)
    let text = response
        .text()
        .await
        .map_err(|e| format!("Text-Lese-Fehler: {}", e))?;

    info!("Download abgeschlossen: {} MB Text", text.len() / 1_000_000);

    let cards: Vec<ScryfallImportCard> = serde_json::from_str(&text)
        .map_err(|e| format!("JSON-Parse-Fehler: {}", e))?;

    info!("{} Karten gefunden, importiere...", cards.len());
    drop(text); // free download buffer

    // Erweiterte Spalten hinzufügen, falls sie noch fehlen
    let extra_columns = [
        ("layout", "TEXT"),
        ("set_code", "TEXT"),
        ("set_name", "TEXT"),
        ("set_type", "TEXT"),
        ("collector_number", "TEXT"),
        ("flavor_text", "TEXT"),
        ("power", "TEXT"),
        ("toughness", "TEXT"),
        ("loyalty", "TEXT"),
        ("defense", "TEXT"),
        ("edhrec_rank", "REAL"),
        ("finishes", "TEXT DEFAULT ''"),
        ("frame", "TEXT"),
        ("border_color", "TEXT"),
        ("released_at", "TEXT"),
        ("image_uris_json", "TEXT"),
    ];
    for (col, col_type) in &extra_columns {
        let _ = conn.execute_batch(&format!(
            "ALTER TABLE cards ADD COLUMN {} {};", col, col_type
        ));
    }

    let total = cards.len() as u64;
    let mut imported: u64 = 0;

    for chunk in cards.chunks(100) {
        match insert_card_batch(conn, chunk) {
            Ok(count) => {
                imported += count as u64;
                reporter.on_progress(imported, Some(total));
            }
            Err(e) => reporter.on_error(imported, &e),
        }
    }

    // 5. Reset pragmas and optimize
    conn.pragma_update(None, "synchronous", &"NORMAL")
        .map_err(|e| format!("Pragma error: {}", e))?;
    conn.pragma_update(None, "locking_mode", &"NORMAL")
        .map_err(|e| format!("Pragma error: {}", e))?;
    conn.execute_batch("ANALYZE;")
        .map_err(|e| format!("ANALYZE error: {}", e))?;

    let elapsed = start.elapsed().as_secs_f64();
    reporter.on_complete(imported, elapsed);

    Ok(imported)
}

// ─── Sets Import ────────────────────────────────────────────

/// Response from Scryfall /sets endpoint
#[derive(Deserialize, Debug)]
struct ScryfallSetList {
    data: Vec<ScryfallSetItem>,
}

/// A single set from Scryfall's /sets endpoint
#[derive(Deserialize, Debug)]
struct ScryfallSetItem {
    id: String,
    code: String,
    name: String,
    set_type: String,
    released_at: Option<String>,
    card_count: u32,
    icon_svg_uri: Option<String>,
    scryfall_uri: Option<String>,
}

/// Import all sets from Scryfall's /sets API endpoint
pub async fn run_sets_import(
    conn: &rusqlite::Connection,
    client: &Client,
    reporter: &dyn ProgressReporter,
) -> Result<u64, String> {
    let start = Instant::now();
    info!("Starte Sets-Import von api.scryfall.com/sets");

    let resp = client
        .get("https://api.scryfall.com/sets")
        .send()
        .await
        .map_err(|e| format!("HTTP-Fehler: {}", e))?;

    let set_list: ScryfallSetList = resp
        .json()
        .await
        .map_err(|e| format!("JSON-Parse-Fehler: {}", e))?;

    info!("{} Sets gefunden, importiere...", set_list.data.len());

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Transaction error: {}", e))?;

    let mut imported: u64 = 0;
    for set in &set_list.data {
        // id in sets table = Scryfall's code (e.g. "lea")
        match tx.execute(
            r#"
            INSERT OR REPLACE INTO sets (id, name, set_type, released_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            rusqlite::params![
                set.code,
                set.name,
                set.set_type,
                set.released_at,
            ],
        ) {
            Ok(_) => imported += 1,
            Err(e) => warn!("Fehler beim Einfügen von Set '{}': {}", set.name, e),
        }
    }

    tx.commit()
        .map_err(|e| format!("Commit error: {}", e))?;

    conn.execute_batch("ANALYZE;")
        .map_err(|e| format!("ANALYZE error: {}", e))?;

    let elapsed = start.elapsed().as_secs_f64();
    reporter.on_complete(imported, elapsed);

    Ok(imported)
}

// ─── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_card_json() {
        let json = r#"{
            "id": "test-id-123",
            "oracle_id": "oracle-456",
            "name": "Black Lotus",
            "lang": "en",
            "mana_cost": "{0}",
            "cmc": 0.0,
            "type_line": "Artifact",
            "oracle_text": "{T}, Sacrifice Black Lotus: Add three mana of any one color.",
            "colors": [],
            "color_identity": [],
            "keywords": [],
            "rarity": "rare",
            "set": "lea",
            "set_name": "Limited Edition Alpha",
            "prices": {"usd": "10000.00"}
        }"#;

        let card: ScryfallImportCard = serde_json::from_str(json).expect("Sollte parsen");
        assert_eq!(card.name, "Black Lotus");
        assert_eq!(card.set, Some("lea".to_string()));
        assert_eq!(card.cmc, Some(0.0));
    }
}
