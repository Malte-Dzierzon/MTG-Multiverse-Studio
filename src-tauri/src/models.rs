//! Models Module - Frontend-facing data structures
//! 
//! These are the types that get serialized and sent to the React frontend via Tauri commands.

use serde::{Deserialize, Serialize};

// ─── DB-INTERNE MODELLE (für SQLite) ──────────────

/// Card as stored in the SQLite database
#[derive(Debug, Clone)]
pub struct CardDb {
    pub id: String,
    pub oracle_id: Option<String>,
    pub name: String,
    pub mana_cost: Option<String>,
    pub cmc: f64,
    pub type_line: String,
    pub oracle_text: Option<String>,
    pub colors: serde_json::Value,
    pub color_identity: serde_json::Value,
    pub keywords: serde_json::Value,
    pub rarity: String,
    pub set_id: String,
    pub image_uris_json: String,
    pub artist: Option<String>,
    pub legalities: String,
    pub prices: String,
}

impl CardDb {
    /// Create a CardDb from raw SQLite row values
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(CardDb {
            id: row.get("id")?,
            oracle_id: row.get("oracle_id")?,
            name: row.get("name")?,
            mana_cost: row.get("mana_cost")?,
            cmc: row.get("cmc")?,
            type_line: row.get("type_line")?,
            oracle_text: row.get("oracle_text")?,
            colors: serde_json::from_str(&row.get::<_, String>("colors")?).unwrap_or_default(),
            color_identity: serde_json::from_str(&row.get::<_, String>("color_identity")?).unwrap_or_default(),
            keywords: serde_json::from_str(&row.get::<_, String>("keywords")?).unwrap_or_default(),
            rarity: row.get("rarity")?,
            set_id: row.get("set_id")?,
            image_uris_json: row.get("image_uris_json")?,
            artist: row.get("artist")?,
            legalities: row.get("legalities")?,
            prices: row.get("prices")?,
        })
    }
}

// ─── FRONTEND-MODELLE (für Tauri Commands) ────────

/// Card response sent to frontend - simplified from ScryfallCard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mana_cost: Option<String>,
    pub cmc: f64,
    pub type_line: String,
    #[serde(rename = "card_text")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_text: Option<String>,
    pub colors: Vec<String>,
    pub color_identity: Vec<String>,
    pub keywords: Vec<String>,
    pub rarity: String,
    pub set: String,
    pub set_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url_small: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url_large: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prices: Option<CardPricesResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legalities: Option<CardLegalitiesResponse>,
}

/// Simplified prices for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardPricesResponse {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub eur: Option<String>,
    pub tix: Option<String>,
}

/// Simplified legalities for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardLegalitiesResponse {
    pub standard: String,
    pub modern: String,
    pub legacy: String,
    pub vintage: String,
    pub commander: String,
    pub pioneer: String,
    pub pauper: String,
}

/// Search result sent to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub cards: Vec<CardResponse>,
    pub total: usize,
    pub from_cache: bool,
}

/// Collection item as stored in the SQLite database
#[derive(Debug, Clone)]
pub struct CollectionItemDb {
    pub id: i64,
    pub card_id: String,
    pub quantity: i32,
    pub condition: String,
    pub notes: Option<String>,
    pub added_at: String,
    pub language: String,
    pub is_foil: bool,
    pub acquired_at: Option<String>,
}

impl CollectionItemDb {
    /// Create from a joined SQLite row (collection + cards)
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(CollectionItemDb {
            id: row.get(0)?,
            card_id: row.get(1)?,
            quantity: row.get(2)?,
            condition: row.get(3)?,
            notes: row.get(4)?,
            added_at: row.get(5)?,
            language: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "en".into()),
            is_foil: row.get::<_, Option<i32>>(7)?.unwrap_or(0) != 0,
            acquired_at: row.get(8)?,
        })
    }
}

/// Collection item sent to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionItemResponse {
    pub id: i64,
    pub card: CardResponse,
    pub quantity: i32,
    pub condition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub added_at: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub is_foil: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquired_at: Option<String>,
}

fn default_language() -> String {
    "en".to_string()
}

/// Paginated collection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub items: Vec<CollectionItemResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

/// Deck response sent to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckResponse {
    pub id: i64,
    pub name: String,
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub cards: Vec<DeckCardResponse>,
}

/// Deck card response (card + quantity + position in deck)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckCardResponse {
    pub card: CardResponse,
    pub quantity: i32,
    pub position: i32,
    pub category: String,
}

/// Created deck response (returned after deck creation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedDeckResponse {
    pub deck: DeckResponse,
}

/// Deck validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckValidationResponse {
    pub valid: bool,
    pub format: String,
    pub issues: Vec<String>,
    pub card_count: i32,
}

/// Lore entry response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreEntryResponse {
    pub id: i64,
    pub title: String,
    pub lore_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub related_cards: Vec<String>,
}

// ─── SETS (Frontend) ─────────────────────────────────

/// Set response sent to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetResponse {
    pub id: String,
    pub name: String,
    pub set_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<String>,
    pub card_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_svg_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scryfall_uri: Option<String>,
}

// ─── COLLECTION IMPORT MODELS ─────────────────────────

/// Statistics returned from a batch collection import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStats {
    pub imported: u64,
    pub updated: u64,
    pub failed: u64,
    pub errors: Vec<String>,
}

/// Result of a collection import command, returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub stats: ImportStats,
}

// ─── PRICE REFRESH MODELS ─────────────────────────

/// Result of a price refresh operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceRefreshResult {
    pub total: u64,
    pub updated: u64,
    pub failed: u64,
    pub errors: Vec<String>,
}

// ─── GOLDFISHING / STARTHAND-SIMULATOR ────────────────

/// Goldfishing / Starthand-Simulator result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldfishResult {
    pub deck_id: i64,
    pub deck_name: String,
    pub starting_hand: Vec<GoldfishCard>,
    pub draws: Vec<Vec<GoldfishCard>>,
    pub turns: i32,
}

/// A card in the goldfishing simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldfishCard {
    pub card_id: String,
    pub name: String,
    pub mana_cost: String,
    pub cmc: f64,
    pub type_line: String,
    pub colors: Vec<String>,
}
