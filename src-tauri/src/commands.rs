//! Tauri Commands - Public API für das Frontend
//!
//! Alle Befehle, die der React-Frontend via invoke() aufrufen kann.

use serde::Deserialize;
use tauri::command;
use tauri::Manager;

use crate::db::{card_repo, collection_repo, deck_repo, lore_repo};
use crate::import_engine::collection_import;
use crate::models::*;
use crate::services::{card_service, deck_service, lore_service, price_service};
use crate::utils::error::{AppError, Result};
use crate::AppState;

// ─── INPUT TYPES (vom Frontend) ────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchCardsArgs {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct GetCardArgs {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct AddToCollectionArgs {
    pub card_id: String,
    pub quantity: Option<i32>,
    pub condition: Option<String>,
    pub language: Option<String>,
    pub is_foil: Option<bool>,
    pub notes: Option<String>,
    pub acquired_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeckArgs {
    pub name: String,
    pub format: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoldfishDeckArgs {
    pub deck_id: i64,
    pub turns: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeckCardQuantityArgs {
    pub deck_id: i64,
    pub card_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReorderDeckCardsArgs {
    pub deck_id: i64,
    pub card_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateDeckArgs {
    pub deck_id: i64,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct LoadLoreArgs {
    pub lore_type: Option<String>,
}

// ─── COMMANDS ──────────────────────────────────────

/// Karten anhand des Namens suchen (lokal + Scryfall-Fallback via Service)
#[command]
pub fn search_cards(
    app: tauri::AppHandle,
    args: SearchCardsArgs,
) -> Result<SearchResult> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let cards = card_service::search_cards_local(&db, &args.query, 50)?;
    let total = cards.len();

    Ok(SearchResult {
        cards,
        total,
        from_cache: true,
    })
}

/// Einzelne Karte per Scryfall-ID abrufen (lokal, später mit API-Fallback)
#[command]
pub fn get_card(
    app: tauri::AppHandle,
    args: GetCardArgs,
) -> Result<CardResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    match card_repo::get_card_by_id(&db, &args.id)? {
        Some(card_db) => Ok(card_repo::card_db_to_response_with_conn(&db, &card_db)),
        None => Err(AppError::NotFound(format!(
            "Karte '{}' nicht in der lokalen Datenbank gefunden",
            args.id
        ))),
    }
}

/// Karte zur Sammlung hinzufügen (via collection_repo)
#[command]
pub fn add_to_collection(
    app: tauri::AppHandle,
    args: AddToCollectionArgs,
) -> Result<CollectionItemResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let card_db = card_repo::get_card_by_id(&db, &args.card_id)?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Karte '{}' nicht gefunden. Bitte zuerst von Scryfall laden.",
                args.card_id
            ))
        })?;

    let quantity = args.quantity.unwrap_or(1);
    let condition = args.condition.unwrap_or_else(|| "nm".to_string());
    let language = args.language.unwrap_or_else(|| "en".to_string());
    let is_foil = args.is_foil.unwrap_or(false);
    let notes = args.notes;
    let acquired_at = args.acquired_at;

    collection_repo::add_to_collection(&db, &args.card_id, quantity, &condition, &language, is_foil, notes.as_deref(), acquired_at.as_deref())?;

    Ok(CollectionItemResponse {
        id: 0,
        card: card_repo::card_db_to_response_with_conn(&db, &card_db),
        quantity,
        condition,
        language,
        is_foil,
        notes,
        acquired_at,
        added_at: chrono::Utc::now().naive_utc().to_string(),
    })
}

/// Neues Deck erstellen (via deck_repo)
#[command]
pub fn create_deck(
    app: tauri::AppHandle,
    args: CreateDeckArgs,
) -> Result<CreatedDeckResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let deck_id = deck_repo::create_deck(
        &db,
        &args.name,
        args.format.as_deref(),
        args.description.as_deref(),
    )?;

    Ok(CreatedDeckResponse {
        deck: deck_service::get_deck_with_cards(&db, deck_id)?,
    })
}

/// Deck inkl. Karten abrufen
#[command]
pub fn get_deck(
    app: tauri::AppHandle,
    args: GetCardArgs,
) -> Result<DeckResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let deck_id: i64 = args.id.parse()
        .map_err(|_| AppError::Validation("Deck-ID muss eine Zahl sein".into()))?;

    deck_service::get_deck_with_cards(&db, deck_id)
}

/// Alle Decks abrufen (mit N+1-fixiertem Batch-Query)
#[command]
pub fn list_decks(
    app: tauri::AppHandle,
) -> Result<Vec<DeckResponse>> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    // Single JOIN query instead of N+1 individual queries
    deck_service::get_all_decks_with_cards_batch(&db)
}

/// Karte zu einem Deck hinzufügen
#[command]
pub fn add_card_to_deck(
    app: tauri::AppHandle,
    deck_id: i64,
    card_id: String,
    quantity: Option<i32>,
    category: Option<String>,
) -> Result<DeckResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let qty = quantity.unwrap_or(1);
    let cat = category.unwrap_or_else(|| "mainboard".to_string());

    let current_cards = deck_repo::get_deck_cards(&db, deck_id)?;
    let next_position = current_cards.iter().map(|c| c.position).max().unwrap_or(-1) + 1;

    deck_repo::add_card_to_deck(&db, deck_id, &card_id, qty, next_position, &cat)?;

    deck_service::get_deck_with_cards(&db, deck_id)
}

/// Karte aus einem Deck entfernen
#[command]
pub fn remove_card_from_deck(
    app: tauri::AppHandle,
    deck_id: i64,
    card_id: String,
) -> Result<DeckResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    deck_repo::remove_card_from_deck(&db, deck_id, &card_id)?;

    deck_service::get_deck_with_cards(&db, deck_id)
}

/// Deck-Metadaten aktualisieren
#[command]
pub fn update_deck(
    app: tauri::AppHandle,
    id: i64,
    name: Option<String>,
    format: Option<String>,
    description: Option<String>,
) -> Result<DeckResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let existing = deck_repo::get_deck_by_id(&db, id)?
        .ok_or_else(|| AppError::NotFound(format!("Deck {} nicht gefunden", id)))?;

    let new_name = name.unwrap_or(existing.name);
    let new_format = format.or(existing.format);
    let new_description = description.or(existing.description);

    deck_repo::update_deck(&db, id, &new_name, new_format.as_deref(), new_description.as_deref())?;

    deck_service::get_deck_with_cards(&db, id)
}

/// Deck löschen
#[command]
pub fn delete_deck(
    app: tauri::AppHandle,
    id: i64,
) -> Result<()> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    deck_repo::delete_deck(&db, id)?;
    Ok(())
}

/// Decks anhand des Namens durchsuchen
#[command]
pub fn search_decks(
    app: tauri::AppHandle,
    query: String,
) -> Result<Vec<DeckResponse>> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let decks = deck_repo::search_decks_by_name(&db, &query)?;
    let mut result = Vec::with_capacity(decks.len());
    for d in decks {
        result.push(deck_service::get_deck_with_cards(&db, d.id)?);
    }
    Ok(result)
}

/// Deck-Legalität gegen ein Format prüfen (erweiterte Prüfung)
#[command]
pub fn validate_deck(app: tauri::AppHandle, id: i64) -> Result<DeckValidationResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let deck = deck_repo::get_deck_by_id(&db, id)?
        .ok_or_else(|| AppError::NotFound(format!("Deck {} nicht gefunden", id)))?;

    let format = deck.format.unwrap_or_else(|| "commander".to_string());

    deck_service::validate_deck_format(&db, id, &format)
}

/// Starthand und Züge simulieren (Goldfishing) für ein Deck
#[command]
pub fn goldfish_deck(
    app: tauri::AppHandle,
    args: GoldfishDeckArgs,
) -> Result<GoldfishResult> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let turns = args.turns.unwrap_or(3);
    deck_service::goldfish_deck(&db, args.deck_id, turns)
}

/// Menge einer Karte im Deck aktualisieren
#[command]
pub fn update_deck_card_quantity(
    app: tauri::AppHandle,
    args: UpdateDeckCardQuantityArgs,
) -> Result<DeckResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    deck_repo::update_deck_card_quantity(&db, args.deck_id, &args.card_id, args.quantity)?;

    deck_service::get_deck_with_cards(&db, args.deck_id)
}

/// Karten in einem Deck neu ordnen (Drag & Drop)
#[command]
pub fn reorder_deck_cards(
    app: tauri::AppHandle,
    args: ReorderDeckCardsArgs,
) -> Result<DeckResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    deck_repo::reorder_deck_cards(&db, args.deck_id, &args.card_ids)?;

    deck_service::get_deck_with_cards(&db, args.deck_id)
}

/// Deck-Legalität gegen ein Format prüfen (explizites Format)
#[command]
pub fn check_deck_legality(
    app: tauri::AppHandle,
    args: ValidateDeckArgs,
) -> Result<DeckValidationResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    deck_service::validate_deck_format(&db, args.deck_id, &args.format)
}

/// Lore-Einträge laden (aus DB + optional aus assets/stories/ Verzeichnis)
#[command]
pub fn load_lore_entries(
    app: tauri::AppHandle,
    args: LoadLoreArgs,
) -> Result<Vec<LoreEntryResponse>> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let entries = lore_repo::get_all_lore_entries(&db, args.lore_type.as_deref())?
        .into_iter()
        .map(|l| LoreEntryResponse {
            id: l.id,
            title: l.title,
            lore_type: l.lore_type,
            content: l.content_path.unwrap_or_default(),
            metadata: serde_json::from_str(&l.metadata).ok(),
            related_cards: serde_json::from_str(&l.related_cards).unwrap_or_default(),
        })
        .collect();

    Ok(entries)
}

/// Mana-Kurve eines Decks berechnen
#[command]
pub fn get_deck_mana_curve(
    app: tauri::AppHandle,
    args: GetCardArgs,
) -> Result<serde_json::Value> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let deck_id: i64 = args.id.parse()
        .map_err(|_| AppError::Validation("Deck-ID muss eine Zahl sein".into()))?;

    let deck_cards = deck_repo::get_deck_cards(&db, deck_id)?;
    let mut cards_with_details = Vec::new();

    for dc in deck_cards {
        if let Some(card_db) = card_repo::get_card_by_id(&db, &dc.card_id)? {
            cards_with_details.push(DeckCardResponse {
                card: card_repo::card_db_to_response_with_conn(&db, &card_db),
                quantity: dc.quantity,
                position: dc.position,
                category: dc.category,
            });
        }
    }

    let curve = deck_service::calculate_mana_curve(&cards_with_details);
    let balance = deck_service::calculate_color_balance(&cards_with_details);

    Ok(serde_json::json!({
        "mana_curve": {
            "0": curve.cmc_0,
            "1": curve.cmc_1,
            "2": curve.cmc_2,
            "3": curve.cmc_3,
            "4": curve.cmc_4,
            "5": curve.cmc_5,
            "6": curve.cmc_6,
            "7+": curve.cmc_7plus,
        },
        "color_balance": {
            "white": balance.white,
            "blue": balance.blue,
            "black": balance.black,
            "red": balance.red,
            "green": balance.green,
            "colorless": balance.colorless,
        }
    }))
}

// ─── SETS COMMANDS ─────────────────────────────────────────

/// Alle Sets aus der lokalen Datenbank abrufen
#[command]
pub fn list_sets(
    app: tauri::AppHandle,
) -> Result<Vec<SetResponse>> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let mut stmt = db.prepare(
        "SELECT id, name, set_type, released_at FROM sets ORDER BY released_at DESC"
    ).map_err(|e| AppError::Database(e.to_string()))?;

    let rows = stmt.query_map([], |row| {
        Ok(SetResponse {
            id: row.get(0)?,
            name: row.get(1)?,
            set_type: row.get(2)?,
            released_at: row.get(3)?,
            card_count: 0,
            icon_svg_uri: None,
            scryfall_uri: None,
        })
    }).map_err(|e| AppError::Database(e.to_string()))?;

    let mut sets = Vec::new();
    for row in rows {
        sets.push(row.map_err(|e| AppError::Database(e.to_string()))?);
    }

    Ok(sets)
}

/// Ein einzelnes Set per Code (id) abrufen
#[command]
pub fn get_set(
    app: tauri::AppHandle,
    args: GetCardArgs,
) -> Result<SetResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let set = db.query_row(
        "SELECT id, name, set_type, released_at FROM sets WHERE id = ?1",
        rusqlite::params![args.id],
        |row| {
            Ok(SetResponse {
                id: row.get(0)?,
                name: row.get(1)?,
                set_type: row.get(2)?,
                released_at: row.get(3)?,
                card_count: 0,
                icon_svg_uri: None,
                scryfall_uri: None,
            })
        },
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Set '{}' nicht gefunden", args.id))
        }
        other => AppError::Database(other.to_string()),
    })?;

    Ok(set)
}

// ─── COLLECTION COMMANDS ─────────────────────────────────

/// Get paginated collection items
#[command]
pub fn get_collection(
    app: tauri::AppHandle,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<CollectionResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let p = page.unwrap_or(1);
    let pp = per_page.unwrap_or(50);

    let total = collection_repo::get_collection_count(&db)?;
    let items_raw = collection_repo::get_collection(&db, p, pp)?;

    let mut items = Vec::with_capacity(items_raw.len());
    for item in items_raw {
        let card_db = card_repo::get_card_by_id(&db, &item.card_id)
            .ok()
            .flatten();
        let card = card_db
            .as_ref()
            .map(|c| card_repo::card_db_to_response_with_conn(&db, c))
            .unwrap_or_default();
        items.push(CollectionItemResponse {
            id: item.id,
            card,
            quantity: item.quantity,
            condition: item.condition,
            notes: item.notes,
            added_at: item.added_at,
            language: item.language,
            is_foil: item.is_foil,
            acquired_at: item.acquired_at,
        });
    }

    Ok(CollectionResponse {
        items,
        total,
        page: p,
        per_page: pp,
    })
}

/// Search collection items by card name
#[command]
pub fn search_collection(
    app: tauri::AppHandle,
    query: String,
) -> Result<Vec<CollectionItemResponse>> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let items_raw = collection_repo::search_collection(&db, &query)?;

    let mut items = Vec::with_capacity(items_raw.len());
    for item in items_raw {
        let card_db = card_repo::get_card_by_id(&db, &item.card_id)
            .ok()
            .flatten();
        let card = card_db
            .as_ref()
            .map(|c| card_repo::card_db_to_response_with_conn(&db, c))
            .unwrap_or_default();
        items.push(CollectionItemResponse {
            id: item.id,
            card,
            quantity: item.quantity,
            condition: item.condition,
            notes: item.notes,
            added_at: item.added_at,
            language: item.language,
            is_foil: item.is_foil,
            acquired_at: item.acquired_at,
        });
    }

    Ok(items)
}

/// Update a collection item's quantity, condition, notes, and acquired_at
#[command]
pub fn update_collection_item(
    app: tauri::AppHandle,
    id: i64,
    quantity: Option<i32>,
    condition: Option<String>,
    notes: Option<String>,
    acquired_at: Option<String>,
) -> Result<CollectionItemResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let existing = collection_repo::get_collection_item(&db, id)?
        .ok_or_else(|| AppError::NotFound(format!("Collection item {} nicht gefunden", id)))?;

    let new_qty = quantity.unwrap_or(existing.quantity);
    let new_cond = condition.unwrap_or(existing.condition);
    let new_acquired_at = acquired_at.or(existing.acquired_at);

    collection_repo::update_collection_item(&db, id, new_qty, &new_cond, notes.as_deref(), new_acquired_at.as_deref())?;

    let updated = collection_repo::get_collection_item(&db, id)?
        .ok_or_else(|| AppError::NotFound("Item disappeared after update".into()))?;

    let card_db = card_repo::get_card_by_id(&db, &updated.card_id)
        .ok()
        .flatten();
    let card = card_db
        .as_ref()
        .map(|c| card_repo::card_db_to_response_with_conn(&db, c))
        .unwrap_or_default();

    Ok(CollectionItemResponse {
        id: updated.id,
        card,
        quantity: updated.quantity,
        condition: updated.condition,
        notes: updated.notes,
        added_at: updated.added_at,
        language: updated.language,
        is_foil: updated.is_foil,
        acquired_at: updated.acquired_at,
    })
}

/// Remove a card from collection
#[command]
pub fn remove_from_collection(
    app: tauri::AppHandle,
    id: i64,
) -> Result<()> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    collection_repo::remove_from_collection(&db, id)?;
    Ok(())
}

/// Import collection data from text (CSV, MTGA, Moxfield, Archidekt)
#[command]
pub fn import_collection(
    app: tauri::AppHandle,
    format: String,
    data: String,
) -> Result<ImportResult> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let parsed = match format.as_str() {
        "csv" => collection_import::parse_csv(&data)?,
        "mtga" => collection_import::parse_mtga(&data)?,
        "moxfield" => collection_import::parse_moxfield_json(&data)?,
        "archidekt" => collection_import::parse_archidekt_json(&data)?,
        _ => return Err(AppError::Validation(format!("Unknown format: {}", format))),
    };

    let stats = collection_repo::import_collection_batch(&db, &parsed)?;

    Ok(ImportResult { stats })
}

// ─── LORE COMMANDS ─────────────────────────────────────────

/// Get a single lore entry with HTML-converted content
#[command]
pub fn get_lore_entry(
    app: tauri::AppHandle,
    id: i64,
) -> Result<LoreEntryResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let lore = lore_repo::get_lore_entry_by_id(&db, id)?
        .ok_or_else(|| AppError::NotFound(format!("Lore-Eintrag {} nicht gefunden", id)))?;

    // Try to read and parse the content file if it exists
    let content_html = if let Some(ref path_str) = lore.content_path {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            match lore_service::parse_lore_file(path) {
                Ok(Some(parsed)) => lore_service::markdown_to_html(&parsed.content),
                _ => String::new(),
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Ok(LoreEntryResponse {
        id: lore.id,
        title: lore.title,
        lore_type: lore.lore_type,
        content: content_html,
        metadata: serde_json::from_str(&lore.metadata).ok(),
        related_cards: serde_json::from_str(&lore.related_cards).unwrap_or_default(),
    })
}

/// Read a lore entry's raw Markdown content from disk and return HTML
#[command]
pub fn get_lore_content(
    app: tauri::AppHandle,
    id: i64,
) -> Result<String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let lore = lore_repo::get_lore_entry_by_id(&db, id)?
        .ok_or_else(|| AppError::NotFound(format!("Lore-Eintrag {} nicht gefunden", id)))?;

    let content_path = lore.content_path
        .ok_or_else(|| AppError::NotFound(format!("Lore-Eintrag {} hat keine Content-Datei", id)))?;

    let path = std::path::Path::new(&content_path);
    let file_content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Io(format!("Fehler beim Lesen von {}: {}", content_path, e)))?;

    Ok(lore_service::markdown_to_html(&file_content))
}

/// Search lore entries by title
#[command]
pub fn search_lore(
    app: tauri::AppHandle,
    query: String,
) -> Result<Vec<LoreEntryResponse>> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let entries = lore_repo::search_lore_entries(&db, &query)?
        .into_iter()
        .map(|l| LoreEntryResponse {
            id: l.id,
            title: l.title,
            lore_type: l.lore_type,
            content: l.content_path.unwrap_or_default(),
            metadata: serde_json::from_str(&l.metadata).ok(),
            related_cards: serde_json::from_str(&l.related_cards).unwrap_or_default(),
        })
        .collect();

    Ok(entries)
}

// ─── PRICE COMMANDS ──────────────────────────────────────

/// Refresh all card prices from Scryfall.
#[command]
pub async fn refresh_prices(
    app: tauri::AppHandle,
) -> Result<PriceRefreshResult> {
    let state = app.state::<AppState>();

    // Get all card IDs (sync, drop guard immediately)
    let card_ids = {
        let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
        card_repo::get_all_card_ids(&db)?
    };

    // Clone Scryfall client (it's Arc-based and thread-safe)
    let scryfall = {
        let client = state.scryfall_client.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
        client.clone()
    };

    // Process in batches
    let mut result = PriceRefreshResult {
        total: card_ids.len() as u64,
        updated: 0,
        failed: 0,
        errors: vec![],
    };

    for chunk in card_ids.chunks(50) {
        match scryfall.get_cards_by_collection(chunk).await {
            Ok(cards) => {
                let mut batch_errors: Vec<String> = Vec::new();
                for card in &cards {
                    let prices_json = if let Some(p) = &card.prices {
                        serde_json::json!({
                            "usd": p.usd,
                            "usd_foil": p.usd_foil,
                            "eur": p.eur,
                            "eur_foil": p.eur_foil,
                            "tix": p.tix,
                        })
                    } else {
                        serde_json::json!({})
                    };

                    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
                    if let Err(e) = card_repo::update_card_prices(&db, &card.id, &prices_json) {
                        batch_errors.push(format!("DB error for {}: {}", card.name, e));
                    } else {
                        result.updated += 1;
                    }
                }
                let missing = chunk.len().saturating_sub(cards.len());
                result.failed += missing as u64;
                result.errors.extend(batch_errors);
            }
            Err(e) => {
                result.failed += chunk.len() as u64;
                result.errors.push(format!("Scryfall batch error: {}", e));
            }
        }

        // Polite delay between batches
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(result)
}

/// Get current prices for a specific card from the database.
#[command]
pub fn get_card_prices(
    app: tauri::AppHandle,
    card_id: String,
) -> Result<CardPricesResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let card = card_repo::get_card_by_id(&db, &card_id)?
        .ok_or_else(|| AppError::NotFound(format!("Karte '{}' nicht gefunden", card_id)))?;

    let prices: serde_json::Value =
        serde_json::from_str(&card.prices).unwrap_or_else(|_| serde_json::json!({}));

    let get_price = |key: &str| -> Option<String> {
        prices
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    };

    Ok(CardPricesResponse {
        usd: get_price("usd"),
        usd_foil: get_price("usd_foil"),
        eur: get_price("eur"),
        tix: get_price("tix"),
    })
}

/// Market Price Response for external APIs (Cardmarket, TCGPlayer)
#[derive(Debug, serde::Serialize)]
pub struct MarketPriceResponse {
    pub source: String,      // "cardmarket" or "tcgplayer"
    pub card_name: String,
    pub set_name: String,
    pub currency: String,
    pub low: Option<f64>,
    pub avg: Option<f64>,
    pub high: Option<f64>,
    pub trend: Option<f64>,
}

/// Fetch a card's market price from Cardmarket (MKM)
#[command]
pub async fn get_cardmarket_price(
    app: tauri::AppHandle,
    card_name: String,
    set_code: String,
) -> Result<Option<MarketPriceResponse>> {
    let state = app.state::<AppState>();
    let client = state.cardmarket_client.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    if !client.is_configured() {
        return Ok(None);
    }

    let price = client.get_price_for_card(&card_name, &set_code).await?;

    Ok(price.map(|p| MarketPriceResponse {
        source: "cardmarket".to_string(),
        card_name: p.card_name,
        set_name: p.set_name,
        currency: p.currency,
        low: p.low,
        avg: p.avg,
        high: p.high,
        trend: p.trend,
    }))
}

/// Fetch a card's market price from TCGPlayer
#[command]
pub async fn get_tcgplayer_price(
    app: tauri::AppHandle,
    card_name: String,
    set_code: String,
) -> Result<Option<MarketPriceResponse>> {
    let state = app.state::<AppState>();
    let mut client = state.tcgplayer_client.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    if !client.is_configured() {
        return Ok(None);
    }

    let price = client.get_market_price(&card_name, &set_code).await?;

    Ok(price.map(|p| MarketPriceResponse {
        source: "tcgplayer".to_string(),
        card_name: p.product_name,
        set_name: "".to_string(), // TCGPlayer doesn't return set name directly
        currency: p.currency,
        low: p.low_price,
        avg: p.market_price,
        high: p.high_price,
        trend: p.mid_price,
    }))
}

/// Refresh prices from ALL sources (Scryfall + Cardmarket + TCGPlayer)
#[command]
pub async fn refresh_all_prices(
    app: tauri::AppHandle,
) -> Result<PriceRefreshResult> {
    let state = app.state::<AppState>();

    // Get all card IDs
    let card_ids = {
        let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
        card_repo::get_all_card_ids(&db)?
    };

    let scryfall = {
        let client = state.scryfall_client.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
        client.clone()
    };

    let cardmarket = {
        let client = state.cardmarket_client.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
        client.clone()
    };

    let mut tcgplayer = {
        let client = state.tcgplayer_client.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
        client.clone()
    };

    let mut result = PriceRefreshResult {
        total: card_ids.len() as u64,
        updated: 0,
        failed: 0,
        errors: vec![],
    };

    for chunk in card_ids.chunks(50) {
        // 1. Scryfall prices
        match scryfall.get_cards_by_collection(chunk).await {
            Ok(cards) => {
                let mut batch_errors: Vec<String> = Vec::new();
                for card in &cards {
                    let prices_json = if let Some(p) = &card.prices {
                        serde_json::json!({
                            "usd": p.usd,
                            "usd_foil": p.usd_foil,
                            "eur": p.eur,
                            "eur_foil": p.eur_foil,
                            "tix": p.tix,
                        })
                    } else {
                        serde_json::json!({})
                    };

                    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
                    if let Err(e) = card_repo::update_card_prices(&db, &card.id, &prices_json) {
                        batch_errors.push(format!("DB error for {}: {}", card.name, e));
                    } else {
                        result.updated += 1;
                    }
                }
                let missing = chunk.len().saturating_sub(cards.len());
                result.failed += missing as u64;
                result.errors.extend(batch_errors);
            }
            Err(e) => {
                result.failed += chunk.len() as u64;
                result.errors.push(format!("Scryfall batch error: {}", e));
            }
        }

        // 2. Cardmarket prices (if configured)
        if cardmarket.is_configured() {
            for card_id in chunk {
                let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
                if let Some(card) = card_repo::get_card_by_id(&db, card_id)? {
                    // Try to get Cardmarket price
                    let set_code = card.set_code.clone();
                    if let Ok(Some(mkt_price)) = cardmarket.get_price_for_card(&card.name, &set_code).await {
                        // Store in price_history table
                        let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
                        let _ = card_repo::insert_price_history(
                            &db,
                            card_id,
                            "cardmarket",
                            "EUR",
                            mkt_price.low,
                            mkt_price.avg,
                            mkt_price.high,
                            mkt_price.trend,
                        );
                    }
                }
            }
        }

        // 3. TCGPlayer prices (if configured)
        if tcgplayer.is_configured() {
            for card_id in chunk {
                let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
                if let Some(card) = card_repo::get_card_by_id(&db, card_id)? {
                    let set_code = card.set_code.clone();
                    if let Ok(Some(mkt_price)) = tcgplayer.get_market_price(&card.name, &set_code).await {
                        let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;
                        let _ = card_repo::insert_price_history(
                            &db,
                            card_id,
                            "tcgplayer",
                            "USD",
                            mkt_price.low_price,
                            mkt_price.market_price,
                            mkt_price.high_price,
                            mkt_price.mid_price,
                        );
                    }
                }
            }
        }

        // Polite delay between batches
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(result)
}