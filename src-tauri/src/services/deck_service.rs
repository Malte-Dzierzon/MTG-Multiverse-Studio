//! Deck Service
//! 
//! Business logic for deck creation, analysis, and statistics.

use std::collections::HashMap;

use crate::db::card_repo;
use crate::db::deck_repo;
use crate::models::{CardResponse, DeckCardResponse, DeckResponse};
use crate::utils::error::{AppError, Result};

/// Mana curve analysis for a deck
#[derive(Debug, Clone)]
pub struct ManaCurve {
    pub cmc_0: i32,
    pub cmc_1: i32,
    pub cmc_2: i32,
    pub cmc_3: i32,
    pub cmc_4: i32,
    pub cmc_5: i32,
    pub cmc_6: i32,
    pub cmc_7plus: i32,
}

/// Color balance analysis for a deck
#[derive(Debug, Clone)]
pub struct ColorBalance {
    pub white: i32,
    pub blue: i32,
    pub black: i32,
    pub red: i32,
    pub green: i32,
    pub colorless: i32,
}

/// Get a deck with all its cards resolved
pub fn get_deck_with_cards(
    conn: &rusqlite::Connection,
    deck_id: i64,
) -> Result<DeckResponse> {
    let deck = deck_repo::get_deck_by_id(conn, deck_id)?
        .ok_or_else(|| AppError::NotFound(format!("Deck {} not found", deck_id)))?;
    
    let deck_cards = deck_repo::get_deck_cards(conn, deck_id)?;
    let cards = resolve_deck_cards(conn, &deck_cards)?;

    Ok(DeckResponse {
        id: deck.id,
        name: deck.name,
        format: deck.format,
        description: deck.description,
        created_at: deck.created_at,
        updated_at: deck.updated_at,
        cards,
    })
}

/// Temporary struct for grouping batch query results
struct DeckCardStub {
    card_id: String,
    quantity: i32,
    position: i32,
    category: String,
}

/// Get all decks with cards in a single batch query (N+1 fix).
/// Groups the joined rows into DeckResponse objects, then resolves card details
/// with deduplicated card fetches.
pub fn get_all_decks_with_cards_batch(
    conn: &rusqlite::Connection,
) -> Result<Vec<DeckResponse>> {
    let rows = deck_repo::get_decks_with_cards_batch(conn)?;

    // --- Phase 1: group joined rows into (Deck, Vec<DeckCardStub>) pairs ---
    struct DeckStub {
        id: i64,
        name: String,
        format: Option<String>,
        description: Option<String>,
        created_at: String,
        updated_at: Option<String>,
        cards: Vec<DeckCardStub>,
    }

    let mut groups: Vec<DeckStub> = Vec::new();
    let mut current_group: Option<DeckStub> = None;

    for row in rows {
        let deck_id = row.deck_id;
        match &mut current_group {
            None => {
                let cards = if let Some(card_id) = row.card_id {
                    vec![DeckCardStub {
                        card_id,
                        quantity: row.quantity.unwrap_or(1),
                        position: row.position.unwrap_or(0),
                        category: row.category.unwrap_or_else(|| "mainboard".to_string()),
                    }]
                } else {
                    vec![]
                };
                current_group = Some(DeckStub {
                    id: deck_id,
                    name: row.deck_name,
                    format: row.deck_format,
                    description: row.deck_description,
                    created_at: row.deck_created_at,
                    updated_at: row.deck_updated_at,
                    cards,
                });
            }
            Some(ref mut g) if g.id == deck_id => {
                // Same deck — add another card stub
                if let Some(card_id) = row.card_id {
                    g.cards.push(DeckCardStub {
                        card_id,
                        quantity: row.quantity.unwrap_or(1),
                        position: row.position.unwrap_or(0),
                        category: row.category.unwrap_or_else(|| "mainboard".to_string()),
                    });
                }
            }
            Some(_) => {
                // Different deck — push current, start new
                let prev = current_group.take().expect("current_group should be Some");
                groups.push(prev);

                let cards = if let Some(card_id) = row.card_id {
                    vec![DeckCardStub {
                        card_id,
                        quantity: row.quantity.unwrap_or(1),
                        position: row.position.unwrap_or(0),
                        category: row.category.unwrap_or_else(|| "mainboard".to_string()),
                    }]
                } else {
                    vec![]
                };
                current_group = Some(DeckStub {
                    id: deck_id,
                    name: row.deck_name,
                    format: row.deck_format,
                    description: row.deck_description,
                    created_at: row.deck_created_at,
                    updated_at: row.deck_updated_at,
                    cards,
                });
            }
        }
    }
    if let Some(g) = current_group.take() {
        groups.push(g);
    }

    // --- Phase 2: resolve all unique card IDs in one pass ---
    let mut all_card_ids: Vec<String> = Vec::new();
    for g in &groups {
        for c in &g.cards {
            if !all_card_ids.contains(&c.card_id) {
                all_card_ids.push(c.card_id.clone());
            }
        }
    }

    let mut card_map: HashMap<String, CardResponse> = HashMap::new();
    for card_id in &all_card_ids {
        if let Some(card_db) = card_repo::get_card_by_id(conn, card_id)? {
            card_map.insert(card_id.clone(), card_repo::card_db_to_response(&card_db));
        }
    }

    // --- Phase 3: build final DeckResponse objects ---
    let mut decks: Vec<DeckResponse> = Vec::with_capacity(groups.len());
    for g in groups {
        let mut cards = Vec::with_capacity(g.cards.len());
        for stub in g.cards {
            let card = card_map
                .remove(&stub.card_id)
                .or_else(|| {
                    // Re-fetch if not in map (shouldn't happen, but be safe)
                    card_repo::get_card_by_id(conn, &stub.card_id)
                        .ok()
                        .and_then(|opt| opt)
                        .map(|db| card_repo::card_db_to_response(&db))
                })
                .unwrap_or_else(|| CardResponse {
                    id: stub.card_id.clone(),
                    name: stub.card_id,
                    mana_cost: None,
                    cmc: 0.0,
                    type_line: String::new(),
                    oracle_text: None,
                    colors: vec![],
                    color_identity: vec![],
                    keywords: vec![],
                    rarity: String::new(),
                    set: String::new(),
                    set_name: String::new(),
                    artist: None,
                    image_url_small: None,
                    image_url_large: None,
                    prices: None,
                    legalities: None,
                });
            cards.push(DeckCardResponse {
                card,
                quantity: stub.quantity,
                position: stub.position,
                category: stub.category,
            });
        }
        decks.push(DeckResponse {
            id: g.id,
            name: g.name,
            format: g.format,
            description: g.description,
            created_at: g.created_at,
            updated_at: g.updated_at,
            cards,
        });
    }

    Ok(decks)
}

/// Resolve DeckCardDb entries into DeckCardResponse with full card details.
fn resolve_deck_cards(
    conn: &rusqlite::Connection,
    deck_cards: &[deck_repo::DeckCardDb],
) -> Result<Vec<DeckCardResponse>> {
    let mut cards = Vec::with_capacity(deck_cards.len());
    for dc in deck_cards {
        if let Some(card_db) = card_repo::get_card_by_id(conn, &dc.card_id)? {
            cards.push(DeckCardResponse {
                card: card_repo::card_db_to_response(&card_db),
                quantity: dc.quantity,
                position: dc.position,
                category: dc.category.clone(),
            });
        }
    }
    Ok(cards)
}

/// Calculate the mana curve for a set of deck cards
pub fn calculate_mana_curve(cards: &[DeckCardResponse]) -> ManaCurve {
    let mut curve = ManaCurve {
        cmc_0: 0, cmc_1: 0, cmc_2: 0, cmc_3: 0,
        cmc_4: 0, cmc_5: 0, cmc_6: 0, cmc_7plus: 0,
    };

    for dc in cards {
        let cmc = dc.card.cmc;
        let qty = dc.quantity;
        match cmc as i32 {
            0 => curve.cmc_0 += qty,
            1 => curve.cmc_1 += qty,
            2 => curve.cmc_2 += qty,
            3 => curve.cmc_3 += qty,
            4 => curve.cmc_4 += qty,
            5 => curve.cmc_5 += qty,
            6 => curve.cmc_6 += qty,
            _ => curve.cmc_7plus += qty,
        }
    }

    curve
}

/// Calculate color balance from deck cards
pub fn calculate_color_balance(cards: &[DeckCardResponse]) -> ColorBalance {
    let mut balance = ColorBalance {
        white: 0, blue: 0, black: 0, red: 0, green: 0, colorless: 0,
    };

    for dc in cards {
        for color in &dc.card.colors {
            let qty = dc.quantity;
            match color.to_lowercase().as_str() {
                "w" => balance.white += qty,
                "u" => balance.blue += qty,
                "b" => balance.black += qty,
                "r" => balance.red += qty,
                "g" => balance.green += qty,
                _ => balance.colorless += qty,
            }
        }
    }

    balance
}
