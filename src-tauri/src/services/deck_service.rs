//! Deck Service
//! 
//! Business logic for deck creation, analysis, and statistics.

use std::collections::HashMap;

use crate::models::{CardResponse, DeckCardResponse, DeckResponse, GoldfishCard, GoldfishResult, DeckValidationResponse};
use crate::db::card_repo;
use crate::db::deck_repo;
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
            Some(g) if g.id == deck_id => {
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

// ─── GOLDFISHING / STARTHAND-SIMULATOR ──────────────

/// Simulate drawing a starting hand and several turns from a deck.
/// Uses the deck's card pool, shuffles randomly, draws 7 for starting hand,
/// then draws 1 card per turn from the remaining pool.
pub fn goldfish_deck(
    conn: &rusqlite::Connection,
    deck_id: i64,
    turns: i32,
) -> Result<GoldfishResult> {
    let deck = deck_repo::get_deck_by_id(conn, deck_id)?
        .ok_or_else(|| AppError::NotFound(format!("Deck {} not found", deck_id)))?;

    let deck_cards = deck_repo::get_deck_cards(conn, deck_id)?;

    // Build a pool with quantity copies of each card
    let mut pool: Vec<GoldfishCard> = Vec::new();
    for dc in &deck_cards {
        if let Some(card_db) = card_repo::get_card_by_id(conn, &dc.card_id)? {
            let colors: Vec<String> =
                serde_json::from_value(card_db.colors.clone()).unwrap_or_default();
            for _ in 0..dc.quantity {
                pool.push(GoldfishCard {
                    card_id: card_db.id.clone(),
                    name: card_db.name.clone(),
                    mana_cost: card_db.mana_cost.clone().unwrap_or_default(),
                    cmc: card_db.cmc,
                    type_line: card_db.type_line.clone(),
                    colors: colors.clone(),
                });
            }
        }
    }

    // Shuffle the pool
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    pool.shuffle(&mut rng);

    // Draw starting hand (up to 7 cards)
    let hand_size = 7.min(pool.len());
    let starting_hand: Vec<GoldfishCard> = pool.drain(..hand_size).collect();

    // Simulate turns: draw 1 card per turn from remaining pool
    let mut draws: Vec<Vec<GoldfishCard>> = Vec::new();
    for _ in 0..turns {
        let mut turn_draws: Vec<GoldfishCard> = Vec::new();
        if !pool.is_empty() {
            turn_draws.push(pool.remove(0));
        }
        draws.push(turn_draws);
    }

    Ok(GoldfishResult {
        deck_id,
        deck_name: deck.name,
        starting_hand,
        draws,
        turns,
    })
}

// ─── ENHANCED FORMAT-LEGALITY CHECK ────────────────

/// Enhanced deck validation against a format.
/// Checks deck size, 4-copy rule (except basic lands), sideboard size,
/// Commander color identity, and per-card format legality.
pub fn validate_deck_format(
    conn: &rusqlite::Connection,
    deck_id: i64,
    format: &str,
) -> Result<DeckValidationResponse> {
    let _deck = deck_repo::get_deck_by_id(conn, deck_id)?
        .ok_or_else(|| AppError::NotFound(format!("Deck {} not found", deck_id)))?;

    let deck_cards = deck_repo::get_deck_cards(conn, deck_id)?;
    let mut issues: Vec<String> = Vec::new();

    // Separate mainboard and sideboard
    let mainboard: Vec<&deck_repo::DeckCardDb> =
        deck_cards.iter().filter(|c| c.category == "mainboard").collect();
    let sideboard: Vec<&deck_repo::DeckCardDb> =
        deck_cards.iter().filter(|c| c.category == "sideboard").collect();

    let mainboard_count: i32 = mainboard.iter().map(|c| c.quantity).sum();
    let sideboard_count: i32 = sideboard.iter().map(|c| c.quantity).sum();

    // ── 1. Deck size check ──────────────────────────────────
    let fmt_lower = format.to_lowercase();
    match fmt_lower.as_str() {
        "commander" => {
            if mainboard_count != 100 {
                issues.push(format!(
                    "Deck hat {} Karten, benötigt genau 100 für Commander",
                    mainboard_count
                ));
            }
        }
        "standard" | "modern" | "pioneer" | "legacy" | "vintage" | "pauper" => {
            if mainboard_count < 60 {
                issues.push(format!(
                    "Deck hat {} Karten, benötigt mindestens 60 für {}",
                    mainboard_count, format
                ));
            }
        }
        "limited" | "draft" | "sealed" => {
            if mainboard_count < 40 {
                issues.push(format!(
                    "Deck hat {} Karten, benötigt mindestens 40 für Limited",
                    mainboard_count
                ));
            }
        }
        _ => {
            // Unknown format — just check minimum 60
            if mainboard_count < 60 {
                issues.push(format!(
                    "Deck hat {} Karten, benötigt mindestens 60",
                    mainboard_count
                ));
            }
        }
    }

    // ── 2. Max 4 copies per card (except basic lands) ──────
    let max_copies: i32 = if fmt_lower == "commander" { 1 } else { 4 };

    for dc in &mainboard {
        if let Some(card_db) = card_repo::get_card_by_id(conn, &dc.card_id)? {
            let is_basic = card_db.type_line.to_lowercase().contains("basic");
            if !is_basic && dc.quantity > max_copies {
                issues.push(format!(
                    "{}: {} Exemplare (max {} erlaubt)",
                    card_db.name, dc.quantity, max_copies
                ));
            }
        }
    }

    // ── 3. Commander-specific checks ──────────────────────
    if fmt_lower == "commander" {
        // Find a legendary creature as commander
        let commander =
            mainboard.iter().find(|dc| {
                if let Some(card_db) =
                    card_repo::get_card_by_id(conn, &dc.card_id).ok().flatten()
                {
                    let tl = card_db.type_line.to_lowercase();
                    (tl.contains("legend") || tl.contains("legendary")) && tl.contains("creature")
                } else {
                    false
                }
            });

        if let Some(cmdr) = commander {
            if let Some(card_db) =
                card_repo::get_card_by_id(conn, &cmdr.card_id).ok().flatten()
            {
                let commander_identity: Vec<String> =
                    serde_json::from_value(card_db.color_identity.clone())
                        .unwrap_or_default();

                // Check all cards fit within commander's color identity
                for dc in &mainboard {
                    if let Some(c) =
                        card_repo::get_card_by_id(conn, &dc.card_id).ok().flatten()
                    {
                        let card_identity: Vec<String> =
                            serde_json::from_value(c.color_identity.clone())
                                .unwrap_or_default();
                        for ci in &card_identity {
                            if !commander_identity.contains(ci) {
                                issues.push(format!(
                                    "{} hat Farbe '{}', die nicht in der Farbidentität des Commanders ({}) liegt",
                                    c.name,
                                    ci,
                                    commander_identity.join(", ")
                                ));
                            }
                        }
                    }
                }
            }
        } else {
            issues.push("Kein legendäres Kreaturen-Commander im Deck gefunden".to_string());
        }
    }

    // ── 4. Sideboard size check ────────────────────────────
    if sideboard_count > 15 {
        issues.push(format!(
            "Sideboard hat {} Karten, maximal 15 erlaubt",
            sideboard_count
        ));
    }

    // ── 5. Per-card format legality (existing logic) ──────
    let illegal_cards = deck_repo::validate_deck_legality(conn, deck_id, format)?;
    for card_name in illegal_cards {
        issues.push(format!("{} ist nicht legal in {}", card_name, format));
    }

    Ok(DeckValidationResponse {
        valid: issues.is_empty(),
        format: format.to_string(),
        card_count: mainboard_count,
        issues,
    })
}

// ─── TESTS ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::init_test_db;

    fn setup_test_db() -> rusqlite::Connection {
        let conn = init_test_db().expect("Failed to create test database");

        // Insert a test set
        conn.execute(
            "INSERT INTO sets (id, code, name, set_type) VALUES ('test', 'TST', 'Test Set', 'core')",
            [],
        )
        .expect("Failed to insert test set");

        // Card 1: Mountain (basic land)
        conn.execute(
            r#"INSERT INTO cards (id, oracle_id, name, mana_cost, cmc, type_line, oracle_text, colors, color_identity, keywords, rarity, set_id, image_uris_json, artist, legalities, prices)
               VALUES ('mountain-1', '', 'Mountain', '', 0.0, 'Basic Land — Mountain', '', '["R"]', '["R"]', '[]', 'common', 'test', '{}', '', '{"standard":"legal","modern":"legal","commander":"legal"}', '{}')"#,
            [],
        )
        .expect("Failed to insert Mountain");

        // Card 2: Island (basic land)
        conn.execute(
            r#"INSERT INTO cards (id, oracle_id, name, mana_cost, cmc, type_line, oracle_text, colors, color_identity, keywords, rarity, set_id, image_uris_json, artist, legalities, prices)
               VALUES ('island-1', '', 'Island', '', 0.0, 'Basic Land — Island', '', '["U"]', '["U"]', '[]', 'common', 'test', '{}', '', '{"standard":"legal","modern":"legal","commander":"legal"}', '{}')"#,
            [],
        )
        .expect("Failed to insert Island");

        // Card 3: Lightning Bolt (non-basic, non-creature)
        conn.execute(
            r#"INSERT INTO cards (id, oracle_id, name, mana_cost, cmc, type_line, oracle_text, colors, color_identity, keywords, rarity, set_id, image_uris_json, artist, legalities, prices)
               VALUES ('bolt-1', '', 'Lightning Bolt', '{R}', 1.0, 'Instant', '', '["R"]', '["R"]', '[]', 'common', 'test', '{}', '', '{"standard":"legal","modern":"legal","commander":"legal"}', '{}')"#,
            [],
        )
        .expect("Failed to insert Lightning Bolt");

        conn
    }

    /// Helper: add quantity copies of a card to mainboard, each at a unique position
    fn add_cards_main(
        conn: &rusqlite::Connection,
        deck_id: i64,
        card_id: &str,
        quantity: i32,
        start_pos: i32,
    ) {
        for i in 0..quantity {
            deck_repo::add_card_to_deck(conn, deck_id, card_id, 1, start_pos + i, "mainboard")
                .expect(&format!("Failed to add {} x{}", card_id, quantity));
        }
    }

    #[test]
    fn test_goldfish_basic_deck() {
        let conn = setup_test_db();

        // Create a deck
        let deck_id = deck_repo::create_deck(&conn, "Test Deck", Some("standard"), None)
            .expect("Failed to create deck");

        // Add 20 Mountains and 20 Islands
        add_cards_main(&conn, deck_id, "mountain-1", 20, 0);
        add_cards_main(&conn, deck_id, "island-1", 20, 20);

        // Goldfish the deck with default 3 turns
        let result = goldfish_deck(&conn, deck_id, 3).expect("Failed to goldfish deck");

        // Starting hand should have exactly 7 cards
        assert_eq!(result.starting_hand.len(), 7, "Starting hand should have 7 cards");
        assert_eq!(result.deck_id, deck_id);
        assert_eq!(result.turns, 3);

        // We should have 3 turns of draws
        assert_eq!(result.draws.len(), 3, "Should have 3 turns of draws");

        // Total cards drawn = 7 (hand) + 3 (turns) = 10
        let total_drawn =
            result.starting_hand.len() + result.draws.iter().map(|d| d.len()).sum::<usize>();
        assert_eq!(total_drawn, 10, "Should have drawn 10 cards total");

        // Deck name should match
        assert_eq!(result.deck_name, "Test Deck");

        // Verify each card is either Mountain or Island
        for card in &result.starting_hand {
            assert!(
                card.name == "Mountain" || card.name == "Island",
                "Unexpected card in starting hand: {}",
                card.name
            );
        }

        // Test with explicit 0 turns
        let result0 = goldfish_deck(&conn, deck_id, 0).expect("Failed with 0 turns");
        assert_eq!(result0.starting_hand.len(), 7);
        assert_eq!(result0.draws.len(), 0);
    }

    #[test]
    fn test_validate_deck_size() {
        let conn = setup_test_db();

        // Create a deck with format = standard
        let deck_id = deck_repo::create_deck(&conn, "Standard Deck", Some("standard"), None)
            .expect("Failed to create deck");

        // Add 4 Mountains + 4 Islands = 8 cards (way below 60 minimum)
        add_cards_main(&conn, deck_id, "mountain-1", 4, 0);
        add_cards_main(&conn, deck_id, "island-1", 4, 4);

        let result = validate_deck_format(&conn, deck_id, "standard")
            .expect("Failed to validate deck");

        assert!(!result.valid, "Deck with 8 cards should not be valid for standard");
        assert!(
            result.issues.iter().any(|i| i.contains("mindestens 60")),
            "Should mention minimum 60 cards, got: {:?}",
            result.issues
        );
        assert_eq!(result.card_count, 8);

        // Test Commander format 100-card requirement
        let cmd_id = deck_repo::create_deck(&conn, "Commander Deck", Some("commander"), None)
            .expect("Failed to create commander deck");
        add_cards_main(&conn, cmd_id, "mountain-1", 1, 0);

        let cmd_result = validate_deck_format(&conn, cmd_id, "commander")
            .expect("Failed to validate commander deck");
        assert!(!cmd_result.valid);
        assert!(
            cmd_result.issues.iter().any(|i| i.contains("genau 100")),
            "Should mention exactly 100 cards, got: {:?}",
            cmd_result.issues
        );
    }

    #[test]
    fn test_validate_four_copy_rule() {
        let conn = setup_test_db();

        let deck_id = deck_repo::create_deck(&conn, "Copy Test", Some("standard"), None)
            .expect("Failed to create deck");

        // 4 Mountains are fine (basic lands exempt)
        add_cards_main(&conn, deck_id, "mountain-1", 14, 0);
        // 4 Islands fine (basic lands exempt)
        add_cards_main(&conn, deck_id, "island-1", 14, 14);
        // Add 5 Lightning Bolts (exceeds max 4 for non-basic)
        add_cards_main(&conn, deck_id, "bolt-1", 5, 28);

        // Total mainboard: 14+14+5 = 33 (still < 60, but let's see the 4-copy issue)
        let result = validate_deck_format(&conn, deck_id, "standard")
            .expect("Failed to validate deck");

        // Should flag the 4-copy violation
        assert!(
            result.issues.iter().any(|i| i.contains("Exemplare") && i.contains("max 4")),
            "Should mention copy limit issue, got: {:?}",
            result.issues
        );
        // Should also mention size < 60
        assert!(
            result.issues.iter().any(|i| i.contains("mindestens 60")),
            "Should mention minimum 60 cards, got: {:?}",
            result.issues
        );
    }

    #[test]
    fn test_validate_sideboard_size() {
        let conn = setup_test_db();

        let deck_id = deck_repo::create_deck(&conn, "Sideboard Test", Some("standard"), None)
            .expect("Failed to create deck");

        // Add enough mainboard (> 60)
        add_cards_main(&conn, deck_id, "mountain-1", 30, 0);
        add_cards_main(&conn, deck_id, "island-1", 30, 30);

        // Add 16 sideboard cards (exceeds max 15)
        for i in 0..16 {
            deck_repo::add_card_to_deck(&conn, deck_id, "bolt-1", 1, 100 + i, "sideboard")
                .expect("Failed to add sideboard bolt");
        }

        let result = validate_deck_format(&conn, deck_id, "standard")
            .expect("Failed to validate deck");

        assert!(
            result.issues.iter().any(|i| i.contains("Sideboard") && i.contains("15")),
            "Should mention sideboard >15, got: {:?}",
            result.issues
        );
    }
}
