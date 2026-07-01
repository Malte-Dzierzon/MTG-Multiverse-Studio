//! Scryfall API Models
//!
//! Data structures for deserializing Scryfall API responses.

use serde::{Deserialize, Serialize};

/// Represents a color in Magic: The Gathering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "W"),
            Color::Blue => write!(f, "U"),
            Color::Black => write!(f, "B"),
            Color::Red => write!(f, "R"),
            Color::Green => write!(f, "G"),
            Color::Colorless => write!(f, "C"),
        }
    }
}

/// Image URIs from Scryfall
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageUris {
    pub small: String,
    pub normal: String,
    pub large: String,
    pub png: Option<String>,
    pub art_crop: Option<String>,
    pub border_crop: Option<String>,
}

/// Legalities object from Scryfall
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Legalities {
    pub standard: String,
    pub future: String,
    pub historic: String,
    pub timeless: String,
    pub gladiator: String,
    pub pioneer: String,
    pub modern: String,
    pub legacy: String,
    pub pauper: String,
    pub vintage: String,
    pub penny: String,
    pub commander: String,
    pub oathbreaker: String,
    #[serde(rename = "standardbrawl")]
    pub standard_brawl: String,
    pub brawl: String,
    #[serde(rename = "competitivebrawl")]
    pub competitive_brawl: String,
    pub alchemy: String,
    #[serde(rename = "paupercommander")]
    pub pauper_commander: String,
    pub duel: String,
    pub oldschool: String,
    pub premodern: String,
    pub predh: String,
    pub tlr: String,
}

/// Prices object from Scryfall
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prices {
    pub usd: Option<String>,
    #[serde(rename = "usd_foil")]
    pub usd_foil: Option<String>,
    #[serde(rename = "usd_etched")]
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    #[serde(rename = "eur_foil")]
    pub eur_foil: Option<String>,
    #[serde(rename = "eur_etched")]
    pub eur_etched: Option<String>,
    pub tix: Option<String>,
}

/// Represents a card from the Scryfall API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScryfallCard {
    pub object: String,
    pub id: String,
    pub oracle_id: Option<String>,
    pub multiverse_ids: Vec<u32>,
    pub mtgo_id: Option<u32>,
    pub mtgo_foil_id: Option<u32>,
    pub tcgplayer_id: Option<u32>,
    pub cardmarket_id: Option<u32>,
    pub name: String,
    pub lang: String,
    #[serde(rename = "released_at")]
    pub released_at: Option<String>,
    pub uri: String,
    pub scryfall_uri: String,
    pub layout: String,
    pub highres_image: bool,
    pub image_status: String,
    #[serde(rename = "mana_cost")]
    pub mana_cost: Option<String>,
    pub cmc: f64,
    #[serde(rename = "type_line")]
    pub type_line: String,
    #[serde(rename = "oracle_text")]
    pub oracle_text: Option<String>,
    pub colors: Vec<Color>,
    pub color_identity: Vec<Color>,
    pub keywords: Vec<String>,
    pub produced_mana: Vec<String>,
    pub loyalty: Option<String>,
    #[serde(rename = "power")]
    pub power: Option<String>,
    #[serde(rename = "toughness")]
    pub toughness: Option<String>,
    pub rarity: String,
    pub set: String,
    pub set_name: String,
    pub set_type: String,
    pub card_back_id: String,
    pub artist: String,
    pub artist_ids: Vec<String>,
    pub illustration_id: String,
    pub border_color: String,
    pub frame: String,
    pub full_art: bool,
    pub textless: bool,
    pub booster: bool,
    pub story_spotlight: bool,
    pub edhrec_rank: Option<u32>,
    pub penny_rank: Option<u32>,
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    pub eur_foil: Option<String>,
    pub eur_etched: Option<String>,
    pub tix: Option<String>,
    #[serde(rename = "related_uris")]
    pub related_uris: Option<serde_json::Value>,
    #[serde(rename = "purchase_uris")]
    pub purchase_uris: Option<serde_json::Value>,
    #[serde(rename = "image_uris")]
    pub image_uris: Option<ImageUris>,
    #[serde(rename = "legalities")]
    pub legalities: Option<Legalities>,
    #[serde(rename = "prices")]
    pub prices: Option<Prices>,
}

/// Represents a set from the Scryfall API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScryfallSet {
    pub object: String,
    pub id: String,
    pub code: String,
    pub mtgo_code: Option<String>,
    pub arena_code: Option<String>,
    pub tcgplayer_id: Option<u32>,
    pub name: String,
    pub set_type: String,
    #[serde(rename = "released_at")]
    pub released_at: Option<String>,
    pub block: Option<String>,
    pub block_code: Option<String>,
    pub parent_set_code: Option<String>,
    pub card_count: u32,
    pub printed_size: u32,
    pub digital: bool,
    pub foil_only: bool,
    pub scryfall_uri: String,
    pub uri: String,
    pub icon_svg_uri: Option<String>,
    pub search_uri: String,
}

/// Wrapper for a list of cards (used in search/paginated results)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScryfallList {
    pub object: String,
    pub total_cards: Option<u32>,
    pub has_more: bool,
    pub next_page: Option<String>,
    pub data: Vec<ScryfallCard>,
    pub warnings: Option<Vec<String>>,
}