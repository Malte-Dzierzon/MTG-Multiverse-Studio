use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub mana_cost: String,
    pub cmc: f64,
    pub type_line: String,
    /// Farbkombination als Rohstring, z.B. "WU"
    pub colors_raw: String,
    pub oracle_text: String,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub flavor_text: Option<String>,
    pub rarity: String,
    pub set_code: String,
    pub artist: String,
    pub prices: HashMap<String, Value>,
    pub legalities: HashMap<String, bool>,
    pub img_normal: Option<String>,
}

fn jmap(v: &str) -> Value {
    serde_json::from_str(v).unwrap_or(Value::Null)
}

impl Card {
    fn from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let imgs = jmap(&r.get::<_, String>(12)?);
        let prices = match jmap(&r.get::<_, String>(11)?) {
            Value::Object(m) => m.into_iter().collect(),
            _ => HashMap::new(),
        };
        let colors_raw = match jmap(&r.get::<_, String>(9)?) {
            Value::Array(a) => a
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(""),
            _ => String::new(),
        };
        let legal_raw = jmap(&r.get::<_, String>(10)?);
        let legalities = legal_raw
            .as_object()
            .map(|m| {
                m.iter()
                    .filter(|(_, v)| v.as_str() == Some("legal"))
                    .map(|(k, _)| (k.clone(), true))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            id: r.get(0)?,
            name: r.get(1)?,
            mana_cost: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            cmc: r.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
            type_line: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
            oracle_text: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
            rarity: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
            colors_raw,
            set_code: r.get::<_, Option<String>>(7)?.unwrap_or_default(),
            artist: r.get::<_, Option<String>>(8)?.unwrap_or_default(),
            prices,
            legalities,
            img_normal: imgs
                .get("large")
                .or_else(|| imgs.get("normal"))
                .or_else(|| imgs.get("small"))
                .and_then(|v| v.as_str())
                .map(String::from),
            power: r.get::<_, Option<String>>(14)?,
            toughness: r.get::<_, Option<String>>(15)?,
            flavor_text: r.get::<_, Option<String>>(16)?,
            loyalty: None, // Will be populated from Scryfall if needed
        })
    }
}

const CARD_COLS: &str = "id, name, mana_cost, cmc, type_line, oracle_text, rarity, \
     set_code, artist, colors, legalities, prices, image_uris_json, set_name, \
     power, toughness, flavor_text";

#[derive(Clone, Debug)]
pub struct DeckEntry {
    pub card: Card,
    pub qty: i64,
    pub category: String,
}

impl DeckEntry {
    pub fn eur(&self) -> f64 {
        self.card
            .prices
            .get("eur")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0)
            * self.qty as f64
    }
}

#[derive(Clone, Debug)]
pub struct Deck {
    pub id: i64,
    pub name: String,
    pub fmt: String,
    pub count: i64,
}

pub struct Db(pub Connection);

impl Db {
    pub fn open(path: &str) -> rusqlite::Result<Self> {
        Ok(Db(Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?))
    }

    // ── search ────────────────────────────────────────────────────

    pub fn search(&self, q: &str, limit: usize) -> Vec<Card> {
        let terms: Vec<&str> = q.split_whitespace().collect();
        let fts = terms
            .iter()
            .map(|t| format!("\"{}\"*", t.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");
        let sql = format!(
            "SELECT {CARD_COLS} FROM cards_fts f JOIN cards c ON c.rowid = f.rowid \
             WHERE cards_fts MATCH ?1 LIMIT ?2"
        );
        let mut cards: Vec<Card> = self
            .0
            .prepare(&sql)
            .and_then(|mut s| {
                s.query_map([&fts, &(limit as i64 * 3).to_string()], Card::from_row)?
                    .collect()
            })
            .unwrap_or_default();

        if cards.is_empty() {
            let like = format!("%{}%", q.trim());
            let sql_like = format!(
                "SELECT {CARD_COLS} FROM cards WHERE name LIKE ?1 COLLATE NOCASE \
                 ORDER BY name LIMIT ?2"
            );
            cards = self
                .0
                .prepare(&sql_like)
                .and_then(|mut s| {
                    s.query_map(
                        rusqlite::params![like, (limit * 3) as i64],
                        Card::from_row,
                    )?
                    .collect()
                })
                .unwrap_or_default();
        }

        let ql = q.trim().to_lowercase();
        let score = |c: &Card| -> (u8, usize) {
            let n = c.name.to_lowercase();
            if n == ql {
                (0, n.len())
            } else if n.starts_with(&ql) || n.split(" // ").next() == Some(ql.as_str()) {
                (1, n.len())
            } else if n.contains(&format!(" {ql}")) {
                (2, n.len())
            } else {
                (3, n.len())
            }
        };
        cards.sort_by_key(&score);
        cards.truncate(limit);
        cards
    }

    pub fn card_by_name(&self, name: &str) -> Option<Card> {
        let sql_exact = format!("SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE");
        let sql_like = format!(
            "SELECT {CARD_COLS} FROM cards WHERE name LIKE ?1 COLLATE NOCASE ORDER BY cmc LIMIT 1"
        );
        self.0
            .prepare(&sql_exact)
            .and_then(|mut s| s.query_row([name], Card::from_row).optional())
            .ok()
            .flatten()
            .or_else(|| {
                self.0
                    .prepare(&sql_like)
                    .and_then(|mut s| {
                        s.query_row([format!("%{name}%")], Card::from_row).optional()
                    })
                    .ok()
                    .flatten()
            })
    }

    /// Erste Karte eines Decks (für Cover).
    pub fn cover_card(&self, deck_id: i64) -> Option<Card> {
        let prefixed = CARD_COLS
            .split(", ")
            .map(|c| format!("c.{c}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT {prefixed} FROM deck_cards dc JOIN cards c ON c.id = dc.card_id \
             WHERE dc.deck_id = ?1 ORDER BY c.cmc DESC LIMIT 1"
        );
        self.0
            .prepare(&sql)
            .and_then(|mut s| s.query_row([deck_id], Card::from_row).optional())
            .ok()
            .flatten()
    }

    // ── decks ─────────────────────────────────────────────────────

    pub fn decks(&self) -> Vec<Deck> {
        self.0
            .prepare(
                "SELECT d.id, d.name, d.format, COALESCE(SUM(dc.quantity),0) \
                 FROM decks d LEFT JOIN deck_cards dc ON dc.deck_id = d.id \
                 GROUP BY d.id \
                 ORDER BY COALESCE(d.updated_at, d.created_at, '') DESC, d.id DESC",
            )
            .and_then(|mut s| {
                s.query_map([], |r| {
                    Ok(Deck {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        fmt: r.get(2)?,
                        count: r.get(3)?,
                    })
                })?
                .collect()
            })
            .unwrap_or_default()
    }

    pub fn deck_create_fmt(&self, name: &str, fmt: &str) -> Deck {
        self.0
            .execute(
                "INSERT INTO decks (name, format, created_at, updated_at) \
                 VALUES (?1, ?2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                rusqlite::params![name, fmt],
            )
            .ok();
        Deck {
            id: self.0.last_insert_rowid(),
            name: name.into(),
            fmt: fmt.into(),
            count: 0,
        }
    }

    pub fn deck_delete(&self, id: i64) {
        let _ = self.0.execute("DELETE FROM deck_cards WHERE deck_id = ?1", [id]);
        let _ = self.0.execute("DELETE FROM decks WHERE id = ?1", [id]);
    }

    pub fn deck_cards(&self, deck_id: i64) -> Vec<DeckEntry> {
        let prefixed = CARD_COLS
            .split(", ")
            .map(|c| format!("c.{c}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT {prefixed}, dc.quantity, dc.category \
             FROM deck_cards dc JOIN cards c ON c.id = dc.card_id \
             WHERE dc.deck_id = ?1 ORDER BY dc.category, c.name"
        );
        let map = |r: &rusqlite::Row<'_>| -> rusqlite::Result<DeckEntry> {
            let card = Card::from_row(r)?;
            Ok(DeckEntry {
                qty: r.get::<_, i64>(17)?,
                category: r
                    .get::<_, Option<String>>(18)?
                    .unwrap_or_else(|| category_of(&card.type_line)),
                card,
            })
        };
        self.0
            .prepare(&sql)
            .and_then(|mut s| {
                s.query_map([deck_id], map)?
                    .collect::<rusqlite::Result<Vec<_>>>()
            })
            .unwrap_or_default()
    }

    pub fn deck_add_card(&self, deck_id: i64, card_id: &str, qty: i64, category: &str) {
        let _ = self.0.execute(
            "INSERT INTO deck_cards (deck_id, card_id, quantity, category) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(deck_id, card_id) DO UPDATE SET quantity = quantity + excluded.quantity",
            rusqlite::params![deck_id, card_id, qty, category],
        );
        let _ = self.0.execute(
            "UPDATE decks SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [deck_id],
        );
    }

    pub fn deck_remove_entry(&self, deck_id: i64, card_id: &str) {
        let _ = self.0.execute(
            "DELETE FROM deck_cards WHERE deck_id = ?1 AND card_id = ?2",
            rusqlite::params![deck_id, card_id],
        );
    }

    // ── import / export ───────────────────────────────────────────

    /// Zeilen parsen: "4 Bolt", "Bolt", Arena "(SET 12)", CSV "Bolt, 4".
    /// Zeile ohne Mengenangabe = qty 1.
    pub fn parse_deck_text(&self, text: &str) -> (Vec<(String, i64)>, Vec<String>) {
        let skip = ["deck", "sideboard", "maybeboard", "commander", ""];
        let mut resolved: HashMap<String, i64> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        let mut missing: Vec<String> = Vec::new();

        for raw in text.lines() {
            let mut line = raw.trim().to_string();
            if skip.contains(&line.to_lowercase().as_str()) {
                continue;
            }
            if !line.starts_with(|c: char| c.is_ascii_digit()) {
                for pre in ["sideboard ", "deck ", "maybeboard "] {
                    if line.to_lowercase().starts_with(pre) {
                        line = line[pre.len()..].trim().to_string();
                        break;
                    }
                }
            }

            let (mut qty, head) = match head_qty(&line) {
                Some((q, rest)) => (q, rest.to_string()),
                None => (1, line.clone()),
            };
            let name = strip_arena(&head);

            let mut card = self.card_by_name(name.trim());
            if card.is_none() {
                // "3 Guide, 2" → Name + zusätzliche Menge
                if let Some((n, extra)) = tail_num(&name) {
                    if let Some(c) = self.card_by_name(n) {
                        qty += extra;
                        card = Some(c);
                    }
                }
            } else if qty == 1 && !line.starts_with(|c: char| c.is_ascii_digit()) {
                // "Guide, 2" → Menge aus CSV-Suffix
                if let Some((n, q)) = tail_num(&name) {
                    if let Some(c) = self.card_by_name(n) {
                        qty = q;
                        card = Some(c);
                    }
                }
            }

            match card {
                Some(c) => match resolved.get_mut(&c.name.to_lowercase()) {
                    Some(v) => *v += qty,
                    None => {
                        resolved.insert(c.name.to_lowercase(), qty);
                        order.push(c.name);
                    }
                },
                None => missing.push(name),
            }
        }
        (
            order
                .into_iter()
                .map(|n| {
                    let q = resolved[&n.to_lowercase()];
                    (n, q)
                })
                .collect(),
            missing,
        )
    }

    pub fn deck_import_text(&self, deck_id: i64, text: &str) -> (usize, Vec<String>) {
        let (pairs, missing) = self.parse_deck_text(text);
        let n = pairs.len();
        for (name, qty) in pairs {
            if let Some(c) = self.card_by_name(&name) {
                self.deck_add_card(deck_id, &c.id, qty, &category_of(&c.type_line));
            }
        }
        (n, missing)
    }

    pub fn deck_export_text(&self, deck_id: i64, name: &str) -> String {
        let cats = [
            "Creatures", "Planeswalkers", "Instants", "Sorceries", "Artifacts",
            "Enchantments", "Lands", "Other",
        ];
        let cards = self.deck_cards(deck_id);
        let total: i64 = cards.iter().map(|e| e.qty).sum();
        let mut out = format!("# {name}\n\n");
        for cat in cats {
            let es: Vec<_> = cards.iter().filter(|e| e.category == cat).collect();
            if es.is_empty() {
                continue;
            }
            out.push_str(&format!("{} ({})\n", cat, es.iter().map(|e| e.qty).sum::<i64>()));
            for e in es {
                out.push_str(&format!("{} {}\n", e.qty, e.card.name));
            }
            out.push('\n');
        }
        out.push_str(&format!("# Total: {total}\n"));
        out
    }
}

pub fn category_of(type_line: &str) -> String {
    let tl = type_line.to_lowercase();
    let table: &[(&str, &str)] = &[
        ("creature", "Creatures"),
        ("planeswalker", "Planeswalkers"),
        ("instant", "Instants"),
        ("sorcery", "Sorceries"),
        ("land", "Lands"),
        ("enchantment", "Enchantments"),
        ("artifact", "Artifacts"),
    ];
    table
        .iter()
        .find(|(k, _)| tl.contains(k))
        .map(|(_, v)| (*v).to_string())
        .unwrap_or_else(|| "Other".into())
}

fn head_qty(line: &str) -> Option<(i64, &str)> {
    let (tok, rest) = line.split_once(char::is_whitespace)?;
    let num = tok.strip_suffix('x').unwrap_or(tok);
    if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
        Some((num.parse().ok()?, rest.trim()))
    } else {
        None
    }
}

fn tail_num(s: &str) -> Option<(&str, i64)> {
    let idx = s.rfind(',')?;
    let n = s[idx + 1..].trim();
    if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) {
        Some((s[..idx].trim(), n.parse().ok()?))
    } else {
        None
    }
}

fn strip_arena(name: &str) -> String {
    if let Some(p) = name.find('(') {
        let tail = name[p..].trim();
        if tail.starts_with('(') && tail.contains(')') {
            return name[..p].trim().to_string();
        }
    }
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn testdb() -> Db {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "mtgtest-{}-{n}.db",
            std::thread::current().name().unwrap_or("t").replace("::", "-")
        ));
        std::fs::copy("assets/mtg_multiverse_studio.db", &tmp).unwrap();
        let db = Db::open(tmp.to_str().unwrap()).unwrap();
        // Test-Decks aufräumen
        db.0.execute("DELETE FROM deck_cards WHERE deck_id IN (SELECT id FROM decks WHERE name LIKE 'T%')", []).ok();
        db.0.execute("DELETE FROM decks WHERE name LIKE 'T%'", []).ok();
        db
    }

    #[test]
    fn parse_qty_prefix() {
        let db = testdb();
        let (pairs, missing) = db.parse_deck_text("4 Lightning Bolt\n");
        assert_eq!(pairs, vec![("Lightning Bolt".into(), 4)]);
        assert!(missing.is_empty());
    }

    #[test]
    fn parse_no_qty_line_must_not_vanish() {
        // Der klassische Bug: Zeilen ohne Menge verschwanden still.
        let db = testdb();
        let (pairs, missing) = db.parse_deck_text("Boseiju, Who Endures\n");
        assert_eq!(pairs, vec![("Boseiju, Who Endures".into(), 1)]);
        assert!(missing.is_empty());
    }

    #[test]
    fn parse_arena_and_csv_and_sideboard() {
        let db = testdb();
        let text = "Deck\n4x Monastery Swiftspear\n1 Ragavan, Nimble Pilferer (MH2) 138\n\
                    Sideboard\n2 Abrade\n3 Goblin Guide, 2\n";
        let (pairs, missing) = db.parse_deck_text(text);
        let m: HashMap<_, _> = pairs.into_iter().collect();
        assert_eq!(m["Monastery Swiftspear"], 4);
        assert_eq!(m["Ragavan, Nimble Pilferer"], 1);
        assert_eq!(m["Abrade"], 2);
        assert_eq!(m["Goblin Guide"], 5); // 3+2 zusammengeführt
        assert!(missing.is_empty(), "{missing:?}");
    }

    #[test]
    fn search_ranks_exact_first() {
        let db = testdb();
        let cards = db.search("lightning bolt", 5);
        assert!(!cards.is_empty());
        assert_eq!(cards[0].name, "Lightning Bolt");
    }

    #[test]
    fn cover_card_has_image_url() {
        let db = testdb();
        let d = db.deck_create_fmt("CoverT", "casual");
        let bolt = db.card_by_name("Lightning Bolt").unwrap();
        assert!(bolt.img_normal.is_some(), "card has no image url!");
        db.deck_add_card(d.id, &bolt.id, 1, "Instants");
        let cover = db.cover_card(d.id).expect("cover card missing");
        assert!(cover.img_normal.as_deref().unwrap().contains("cards.scryfall.io"));
    }

    #[test]
    fn deck_crud_roundtrip() {
        let db = testdb();
        let d = db.deck_create_fmt("TCRUD", "casual");
        let bolt = db.card_by_name("Lightning Bolt").unwrap();
        db.deck_add_card(d.id, &bolt.id, 4, "Instants");
        let entries = db.deck_cards(d.id);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].qty, 4);
        db.deck_add_card(d.id, &bolt.id, 2, "Instants");
        assert_eq!(db.deck_cards(d.id)[0].qty, 6);
        db.deck_remove_entry(d.id, &bolt.id);
        assert!(db.deck_cards(d.id).is_empty());
        db.deck_delete(d.id);
        assert!(!db.decks().iter().any(|x| x.name == "TCRUD"));
    }
}
