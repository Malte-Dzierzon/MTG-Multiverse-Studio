//! Collection Import Parser Module
//!
//! Parses collection data from various external formats:
//! - CSV (name, quantity, condition, language, is_foil, set_code)
//! - MTGArena export format
//! - Moxfield JSON deck export
//! - Archidekt JSON deck export
//!
//! Returns Vec<CollectionImportItem> for processing by collection_repo.

use std::fmt;

/// A single item to import into the collection
#[derive(Debug, Clone)]
pub struct CollectionImportItem {
    pub card_identifier: String,
    pub quantity: i32,
    pub condition: String,
    pub language: String,
    pub is_foil: bool,
    pub notes: Option<String>,
    pub acquired_at: Option<String>,
}

/// Simple error type for collection import parsing
#[derive(Debug)]
pub struct ImportError(String);

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Import error: {}", self.0)
    }
}

impl std::error::Error for ImportError {}

/// Convenience alias for import parsing results
pub type Result<T> = std::result::Result<T, ImportError>;

/// Parse CSV text into collection import items.
///
/// Expected columns (with or without header row):
///   name,quantity,condition,language,is_foil,set_code
///
/// All columns except `name` are optional.
pub fn parse_csv(text: &str) -> Result<Vec<CollectionImportItem>> {
    let mut items = Vec::new();

    for (line_num, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();
        if fields.is_empty() || fields[0].is_empty() {
            continue;
        }

        // Skip header row
        if line_num == 0 && fields[0].to_lowercase() == "name" {
            continue;
        }

        let name = fields[0].to_string();
        let quantity = fields
            .get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);
        let condition = fields
            .get(2)
            .filter(|s| !s.is_empty())
            .unwrap_or(&"nm")
            .to_string();
        let language = fields
            .get(3)
            .filter(|s| !s.is_empty())
            .unwrap_or(&"en")
            .to_string();
        let is_foil = fields
            .get(4)
            .map(|s| matches!(*s, "1" | "true" | "yes" | "foil" | "y"))
            .unwrap_or(false);
        // field[5] = set_code (ignored — used only for resolution hints)
        // field[6] = notes (optional)
        // field[7] = acquired_at (optional)
        let notes = fields
            .get(6)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let acquired_at = fields
            .get(7)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        items.push(CollectionImportItem {
            card_identifier: name,
            quantity,
            condition,
            language,
            is_foil,
            notes,
            acquired_at,
        });
    }

    Ok(items)
}

/// Parse MTG Arena export format.
///
/// Format: `1 Card Name (SET) 123`
/// Each line: `<quantity> <card name> (<set code>) <collector number>`
///
/// The set code and collector number are optional.
pub fn parse_mtga(text: &str) -> Result<Vec<CollectionImportItem>> {
    let mut items = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Try to match: quantity + name + optional (SET) + optional number
        let trimmed = line;
        let quantity = if let Some(first_space) = trimmed.find(' ') {
            let q_str = &trimmed[..first_space];
            q_str.parse::<i32>().unwrap_or(1)
        } else {
            1
        };

        // Extract the card name by stripping suffix
        let rest = if let Some(first_space) = trimmed.find(' ') {
            &trimmed[first_space + 1..]
        } else {
            trimmed
        };

        // Strip trailing (SET) and collector number
        let name = rest
            .split(" (")
            .next()
            .unwrap_or(rest)
            .trim()
            .to_string();

        // Detect foil: MTGA format doesn't explicitly mark foil in export,
        // but some exports have "F" suffix or similar — we default to false.
        let is_foil = false;

        if !name.is_empty() {
            items.push(CollectionImportItem {
                card_identifier: name,
                quantity,
                condition: "nm".to_string(),
                language: "en".to_string(),
                is_foil,
                notes: None,
                acquired_at: None,
            });
        }
    }

    Ok(items)
}

/// Parse Moxfield JSON deck export.
///
/// Expected structure:
/// ```json
/// {
///   "boards": {
///     "mainboard": [
///       { "card": { "name": "...", "set": "...", ... }, "quantity": 1 }
///     ],
///     "sideboard": [...],
///     "maybeboard": [...]
///   }
/// }
/// ```
///
/// Also handles newer Moxfield format where cards are objects keyed by card name:
/// ```json
/// {
///   "mainboard": {
///     "Card Name": { "quantity": 1, "board": "...", ... }
///   }
/// }
/// ```
pub fn parse_moxfield_json(json: &str) -> Result<Vec<CollectionImportItem>> {
    use serde_json::Value;

    let root: Value =
        serde_json::from_str(json).map_err(|e| ImportError(format!("Moxfield parse error: {}", e)))?;

    let mut items = Vec::new();

    // Determine structure: "boards" wrapper or flat?
    if let Some(boards) = root.get("boards") {
        // Newer format: boards.mainboard is an array of {card: {name}, quantity}
        for board_name in &["mainboard", "sideboard", "maybeboard"] {
            if let Some(board) = boards.get(*board_name) {
                extract_moxfield_cards(board, &mut items);
            }
        }
    } else {
        // Flat format: keys = board names directly
        for board_name in &["mainboard", "sideboard", "maybeboard"] {
            if let Some(board) = root.get(*board_name) {
                extract_moxfield_cards(board, &mut items);
            }
        }
    }

    Ok(items)
}

/// Extract cards from a Moxfield board section (handles both array and object formats)
fn extract_moxfield_cards(board: &serde_json::Value, items: &mut Vec<CollectionImportItem>) {
    use serde_json::Value;

    match board {
        Value::Array(arr) => {
            // Format: [{card: {name, ...}, quantity: N}, ...]
            for entry in arr {
                let name = entry
                    .get("card")
                    .and_then(|c| c.get("name"))
                    .and_then(|n| n.as_str())
                    .map(String::from);
                let quantity = entry
                    .get("quantity")
                    .and_then(|q| q.as_i64())
                    .unwrap_or(1) as i32;
                let is_foil = entry
                    .get("card")
                    .and_then(|c| c.get("finishes"))
                    .and_then(|f| f.as_array())
                    .map(|finishes| finishes.iter().any(|f| f.as_str() == Some("foil")))
                    .unwrap_or(false);

                if let Some(name) = name {
                    items.push(CollectionImportItem {
                        card_identifier: name,
                        quantity,
                        condition: "nm".to_string(),
                        language: "en".to_string(),
                        is_foil,
                        notes: None,
                        acquired_at: None,
                    });
                }
            }
        }
        Value::Object(map) => {
            // Format: {"Card Name": {quantity: N, ...}, ...}
            for (card_name, entry) in map {
                let quantity = entry
                    .get("quantity")
                    .and_then(|q| q.as_i64())
                    .unwrap_or(1) as i32;
                let is_foil = entry
                    .get("finishes")
                    .and_then(|f| f.as_array())
                    .map(|finishes| finishes.iter().any(|f| f.as_str() == Some("foil")))
                    .or_else(|| entry.get("isFoil").and_then(|f| f.as_bool()))
                    .unwrap_or(false);

                items.push(CollectionImportItem {
                    card_identifier: card_name.clone(),
                    quantity,
                    condition: "nm".to_string(),
                    language: "en".to_string(),
                    is_foil,
                    notes: None,
                    acquired_at: None,
                });
            }
        }
        _ => {}
    }
}

/// Parse Archidekt JSON deck export.
///
/// Expected structure:
/// ```json
/// {
///   "name": "My Deck",
///   "cards": [
///     {
///       "card": {
///         "oracleCard": { "name": "..." },
///         "edition": { "editionCode": "..." }
///       },
///       "quantity": 1
///     }
///   ]
/// }
/// ```
///
/// Also handles the alternative flat format:
/// ```json
/// {
///   "name": "My Deck",
///   "cards": [
///     {
///       "card": { "name": "...", "set": "..." },
///       "quantity": 1
///     }
///   ]
/// }
/// ```
pub fn parse_archidekt_json(json: &str) -> Result<Vec<CollectionImportItem>> {
    use serde_json::Value;

    let root: Value = serde_json::from_str(json)
        .map_err(|e| ImportError(format!("Archidekt parse error: {}", e)))?;

    let mut items = Vec::new();

    if let Some(cards) = root.get("cards").and_then(|c| c.as_array()) {
        for entry in cards {
            let quantity = entry
                .get("quantity")
                .and_then(|q| q.as_i64())
                .unwrap_or(1) as i32;

            let card_val = match entry.get("card") {
                Some(c) => c,
                None => continue,
            };

            // Try oracleCard.name, then direct name, then card.name
            let name = card_val
                .get("oracleCard")
                .and_then(|oc| oc.get("name"))
                .and_then(|n| n.as_str())
                .or_else(|| card_val.get("name").and_then(|n| n.as_str()))
                .map(String::from);

            // Detect foil: check finishes or isFoil
            let is_foil = entry
                .get("finishes")
                .and_then(|f| f.as_array())
                .map(|finishes| finishes.iter().any(|f| f.as_str() == Some("foil")))
                .or_else(|| {
                    entry
                        .get("isFoil")
                        .and_then(|f| f.as_bool())
                })
                .unwrap_or(false);

            if let Some(name) = name {
                items.push(CollectionImportItem {
                    card_identifier: name,
                    quantity,
                    condition: "nm".to_string(),
                    language: "en".to_string(),
                    is_foil,
                    notes: None,
                    acquired_at: None,
                });
            }
        }
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_simple() {
        let csv = "Black Lotus,1,nm,en,1,lea\nIsland,10,mp,en,0\n";
        let items = parse_csv(csv).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].card_identifier, "Black Lotus");
        assert_eq!(items[0].quantity, 1);
        assert_eq!(items[0].is_foil, true);
        assert_eq!(items[1].card_identifier, "Island");
        assert_eq!(items[1].quantity, 10);
        assert_eq!(items[1].is_foil, false);
    }

    #[test]
    fn test_parse_csv_with_header() {
        let csv = "name,quantity,condition,language,is_foil,set_code\nBlack Lotus,1,nm,en,1,lea\n";
        let items = parse_csv(csv).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].card_identifier, "Black Lotus");
    }

    #[test]
    fn test_parse_mtga() {
        let text = "1 Black Lotus (LEA) 123\n4 Counterspell (2ED) 45\n";
        let items = parse_mtga(text).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].card_identifier, "Black Lotus");
        assert_eq!(items[0].quantity, 1);
        assert_eq!(items[1].card_identifier, "Counterspell");
        assert_eq!(items[1].quantity, 4);
    }

    #[test]
    fn test_parse_moxfield_json() {
        let json = r#"{
            "boards": {
                "mainboard": [
                    { "card": { "name": "Black Lotus", "set": "lea", "finishes": ["nonfoil"] }, "quantity": 1 },
                    { "card": { "name": "Island", "set": "lea", "finishes": ["foil"] }, "quantity": 10 }
                ],
                "sideboard": [
                    { "card": { "name": "Counterspell", "set": "2ed" }, "quantity": 2 }
                ]
            }
        }"#;
        let items = parse_moxfield_json(json).unwrap();
        assert_eq!(items.len(), 3);
        assert!(!items[0].is_foil);
        assert!(items[1].is_foil);
        assert_eq!(items[2].card_identifier, "Counterspell");
    }

    #[test]
    fn test_parse_archidekt_json() {
        let json = r#"{
            "name": "Test Deck",
            "cards": [
                { "card": { "oracleCard": { "name": "Black Lotus" } }, "quantity": 1 },
                { "card": { "oracleCard": { "name": "Island" } }, "quantity": 10, "finishes": ["foil"] }
            ]
        }"#;
        let items = parse_archidekt_json(json).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].card_identifier, "Black Lotus");
        assert_eq!(items[0].quantity, 1);
        assert_eq!(items[1].card_identifier, "Island");
        assert!(items[1].is_foil);
    }

    #[test]
    fn test_parse_empty_inputs() {
        assert!(parse_csv("").unwrap().is_empty());
        assert!(parse_mtga("").unwrap().is_empty());
        assert!(parse_mtga("  \n  \n").unwrap().is_empty());
    }

    #[test]
    fn test_parse_moxfield_object_format() {
        // Newer Moxfield format where cards are keyed by name
        let json = r#"{
            "mainboard": {
                "Black Lotus": { "quantity": 1 },
                "Island": { "quantity": 10, "isFoil": true }
            }
        }"#;
        let items = parse_moxfield_json(json).unwrap();
        assert_eq!(items.len(), 2);
        assert!(items[1].is_foil);
    }

    #[test]
    fn test_parse_mtga_no_suffix() {
        let text = "1 Black Lotus\n4 Counterspell\n";
        let items = parse_mtga(text).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].card_identifier, "Black Lotus");
        assert_eq!(items[0].quantity, 1);
    }

    #[test]
    fn test_parse_archidekt_flat_format() {
        let json = r#"{
            "name": "Test",
            "cards": [
                { "card": { "name": "Black Lotus", "set": "lea" }, "quantity": 1 }
            ]
        }"#;
        let items = parse_archidekt_json(json).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].card_identifier, "Black Lotus");
    }
}
