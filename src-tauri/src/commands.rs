//! Tauri Commands - Public API für das Frontend
//!
//! Alle Befehle, die der React-Frontend via invoke() aufrufen kann.

use serde::Deserialize;
use tauri::command;
use tauri::Manager;

use crate::db::{card_repo, collection_repo, deck_repo, lore_repo};
use crate::models::*;
use crate::services::{card_service, deck_service, lore_service};
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
}

#[derive(Debug, Deserialize)]
pub struct CreateDeckArgs {
    pub name: String,
    pub format: Option<String>,
    pub description: Option<String>,
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

    collection_repo::add_to_collection(&db, &args.card_id, quantity, &condition)?;

    Ok(CollectionItemResponse {
        id: 0,
        card: card_repo::card_db_to_response_with_conn(&db, &card_db),
        quantity,
        condition,
        notes: None,
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

/// Deck-Legalität gegen ein Format prüfen
#[command]
pub fn validate_deck(
    app: tauri::AppHandle,
    id: i64,
) -> Result<DeckValidationResponse> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| AppError::Unknown(e.to_string()))?;

    let deck = deck_repo::get_deck_by_id(&db, id)?
        .ok_or_else(|| AppError::NotFound(format!("Deck {} nicht gefunden", id)))?;

    let format = deck.format.unwrap_or_else(|| "commander".to_string());
    let card_count = deck_repo::count_deck_cards(&db, id)?;
    let issues = deck_repo::validate_deck_legality(&db, id, &format)?;

    Ok(DeckValidationResponse {
        valid: issues.is_empty(),
        format,
        card_count,
        issues,
    })
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
