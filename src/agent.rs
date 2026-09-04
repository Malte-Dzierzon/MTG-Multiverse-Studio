//! Agent-Harness: OpenCode Zen (OpenAI-kompatibel), Kontext-Protokoll,
//! Skills/Pins/Notizen mit Persistenz.

use serde_json::{json, Value};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

pub const DEFAULT_BASE_URL: &str = "https://opencode.ai/zen/v1";
pub const DEFAULT_MODEL: &str = "x-preview-f-free"; // Free-Tier, Zero-Retention
const AUTH_JSON: &str = ".local/share/opencode/auth.json";
const DEFAULT_STORE_PATH: &str = ".mtg-tui.json";

fn store_path() -> String {
    std::env::var("MTG_TUI_STORE").unwrap_or_else(|_| DEFAULT_STORE_PATH.into())
}

#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl AgentConfig {
    /// Key-Kette: MTG_LLM_API_KEY → .env → OpenCode auth.json.
    pub fn from_env() -> Self {
        load_env(".env");
        let home = std::env::var("HOME").unwrap_or_default();
        let api_key = std::env::var("MTG_LLM_API_KEY")
            .ok()
            .filter(|k| !k.is_empty())
            .or_else(|| {
                std::fs::read_to_string(format!("{home}/{AUTH_JSON}"))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .and_then(|v| {
                        v["opencode"]["key"]
                            .as_str()
                            .map(std::string::ToString::to_string)
                    })
            })
            .unwrap_or_default();
        Self {
            api_key,
            model: std::env::var("MTG_LLM_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into()),
            base_url: std::env::var("MTG_LLM_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_BASE_URL.into()),
        }
    }

    pub fn ready(&self) -> bool {
        !effective_key(self).is_empty()
    }
}

/// Zur Laufzeit gesetzter Key (Settings-Screen), überschreibt cfg.api_key.
static RUNTIME_KEY: Mutex<Option<String>> = Mutex::new(None);

pub fn set_runtime_key(key: String) {
    *RUNTIME_KEY.lock().unwrap() = if key.is_empty() { None } else { Some(key) };
}

fn effective_key(cfg: &AgentConfig) -> String {
    RUNTIME_KEY
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| cfg.api_key.clone())
}

fn env_path() -> String {
    std::env::var("MTG_TUI_ENV").unwrap_or_else(|_| ".env".into())
}

/// Key in der .env-Datei persistieren (Zeile ersetzen oder anhängen).
pub fn save_key_to_env(key: &str) -> std::io::Result<()> {
    let path = env_path();
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let line = format!("MTG_LLM_API_KEY={key}");
    let mut out = String::new();
    let mut replaced = false;
    for l in existing.lines() {
        if l.trim_start().starts_with("MTG_LLM_API_KEY=") {
            out.push_str(&line);
            replaced = true;
        } else {
            out.push_str(l);
        }
        out.push('\n');
    }
    if !replaced {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&line);
        out.push('\n');
    }
    std::fs::write(&path, out)
}

fn load_env(path: &str) {
    let Ok(text) = std::fs::read_to_string(path) else { return };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if std::env::var(key).is_err() {
                std::env::set_var(key.trim(), value.trim().trim_matches('"'));
            }
        }
    }
}

pub const SYSTEM_PROMPT: &str = "Du bist ein erfahrener Magic: The Gathering Deckbau-Coach \
und arbeitest in einem Terminal-Tool. Du erhältst optional Kontext: das aktuell geöffnete \
Deck (Menge x Karte – Kosten | Typ | Set | EUR), angeheftete Karten und Notizen.

Arbeitsweise:
- Analysiere konkret statt allgemein: Manakurve (CMC-Verteilung), Landanteil (~24 bei 60 \
Karten, ~17 bei 40), Farben/Ramp, Rollenabdeckung (Removal, Card Advantage, Winconditions), \
Synergien zwischen vorhandenen Karten, Format-Legalities.
- Beziehe dich bei Vorschlägen immer auf konkrete Karten im Deck und nenne den Grund.
- Nutze nur echte, existierende Karten mit exakter offizieller Schreibweise.
- Antworte kompakt in Stichpunkten. Keine Einleitung, keine Floskeln, kein Smalltalk.

MTG-Wissen:
- Kenne die gängigen Formate: Standard, Modern, Pioneer, Legacy, Commander, Pauper, Vintage
- Verstehe Manakurve: CMC 0-1 (Ramp/Mana), 2 (Effizient), 3-4 (Midrange), 5+ (Finisher)
- Rollen: Removal (Bolt, Path), Card Advantage (Brainstorm, Rhystic Study), Wincon (Combo, Beatdown)
- Mana-Basis: Dual Lands, Fetch Lands, Shock Lands, Basics, utility Lands
- Synergien: Engine Cards, Combo Pieces, Tribal Support, Graveyard Interaktion

Karten-Recherche:
Wenn du aktuelle Kartendaten brauchst (Preise, Drucke, Combos, Legality), antworte NUR mit:
[[search: scryfall suchanfrage]]
Du erhältst die Treffer und schließt dann deine eigentliche Antwort an.

Precon-Decks:
Wenn der Nutzer nach vorgefertigten Decks sucht, nutze:
[[search: is:precon format-name]]
z.B. [[search: is:precon commander]] für Commander Precons.

Deck-Änderungen:
Wenn du Karten hinzufügen oder entfernen willst, füge am Ende genau einen JSON-Block hinzu:
```json
{\"add\": [[\"Exakter Kartenname\", menge]], \"remove\": [\"Exakter Kartenname\"]}
```
\"add\" erhöht die Menge um die Anzahl (Default 1); \"remove\" entfernt ALLE Exemplare. \
Ohne Änderungswunsch kein JSON.";

// ── Persistenter Store: Pins, Notizen, Skills ────────────────────

#[derive(Clone, Debug)]
pub struct Skill {
    pub name: String,
    pub desc: &'static str,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Store {
    pub pinned: Vec<String>,
    pub notes: Vec<String>,
    pub skills: Vec<Skill>,
}

impl Store {
    pub fn load() -> Self {
        let mut s = Store {
            skills: vec![
                Skill { name: "context".into(), desc: "Deckkontext mitsenden", enabled: true },
                Skill { name: "pins".into(), desc: "Angeheftete Karten mitsenden", enabled: true },
                Skill { name: "notes".into(), desc: "Notizen mitsenden", enabled: true },
                Skill { name: "scryfall".into(), desc: "[[search: …]] Web-Recherche", enabled: false },
            ],
            ..Default::default()
        };
        let Ok(text) = std::fs::read_to_string(store_path()) else { return s };
        let Ok(v) = serde_json::from_str::<Value>(&text) else { return s };
        if let Some(arr) = v["pinned"].as_array() {
            s.pinned = arr.iter().filter_map(|x| x.as_str().map(String::from)).collect();
        }
        if let Some(arr) = v["notes"].as_array() {
            s.notes = arr.iter().filter_map(|x| x.as_str().map(String::from)).collect();
        }
        for sv in v["skills"].as_array().into_iter().flatten() {
            if let (Some(name), Some(on)) = (sv["name"].as_str(), sv["enabled"].as_bool()) {
                if let Some(skill) = s.skills.iter_mut().find(|k| k.name == name) {
                    skill.enabled = on;
                }
            }
        }
        s
    }

    pub fn save(&self) {
        let v = json!({
            "pinned": self.pinned,
            "notes": self.notes,
            "skills": self.skills.iter().map(|k| json!({"name": k.name, "enabled": k.enabled}))
                .collect::<Vec<_>>(),
        });
        let _ = serde_json::to_string_pretty(&v)
            .map(|s| std::fs::write(store_path(), s));
    }

    pub fn toggle(&mut self, name: &str) -> bool {
        // true wenn gefunden+getoggelt
        if let Some(k) = self.skills.iter_mut().find(|k| k.name == name) {
            k.enabled = !k.enabled;
            self.save();
            return true;
        }
        false
    }

    pub fn pin_toggle(&mut self, name: &str) -> bool {
        match self.pinned.iter().position(|p| p == name) {
            Some(i) => {
                self.pinned.remove(i);
                self.save();
                false
            }
            None => {
                self.pinned.push(name.to_string());
                self.save();
                true
            }
        }
    }

    pub fn add_note(&mut self, note: String) {
        self.notes.push(note);
        self.save();
    }
}

// ── Chat-Worker ──────────────────────────────────────────────────

/// Eine Nachricht im Chatverlauf: (role, content).
pub type Msg = (String, String);

pub enum Out {
    Reply(String),
    Failed(String),
}

static TX: Mutex<Option<Sender<Vec<Msg>>>> = Mutex::new(None);

/// Frage stellen (History ohne System-Prompt).
pub fn ask(history: Vec<Msg>) {
    if let Ok(guard) = TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(history);
        }
    }
}

/// Scryfall-Fulltext-Suche (für den [[search: …]]-Skill).
type FollowupCtx = Vec<Msg>;
static SCRY_TX: Mutex<Option<Sender<(String, FollowupCtx)>>> = Mutex::new(None);

#[derive(Debug)]
pub struct ScryfallHit {
    pub name: String,
    pub mana_cost: String,
    pub type_line: String,
    pub eur: Option<String>,
}

pub fn scryfall_search(query: String, followup_ctx: Vec<Msg>) {
    if let Ok(guard) = SCRY_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send((query, followup_ctx));
        }
    }
}

pub fn spawn_scryfall_worker() -> Receiver<(String, Vec<ScryfallHit>)> {
    let (tx_out, rx_out) = std::sync::mpsc::channel();
    let (tx_in, rx_in) = std::sync::mpsc::channel::<(String, Vec<Msg>)>();
    *SCRY_TX.lock().unwrap() = Some(tx_in);
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(15))
            .build();
        for (query, _ctx) in rx_in {
            let hits = scryfall_query(&agent, &query).unwrap_or_default();
            let _ = tx_out.send((query, hits));
        }
    });
    rx_out
}

fn scryfall_query(agent: &ureq::Agent, query: &str) -> Option<Vec<ScryfallHit>> {
    let q: String = query
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || "-:\"'".contains(*c))
        .collect();
    let url = format!(
        "https://api.scryfall.com/cards/search?q={}",
        q.replace(' ', "%20")
    );
    let body: Value = agent.get(&url).call().ok()?.into_json().ok()?;
    let mut out = Vec::new();
    for card in body["data"].as_array()?.iter().take(8) {
        // Doppelseitige Karten haben image_uris auf card_faces[0]
        let name = card["name"].as_str()?.to_string();
        let cost = card["mana_cost"]
            .as_str()
            .or_else(|| card["card_faces"][0]["mana_cost"].as_str())
            .unwrap_or("")
            .to_string();
        let tl = card["type_line"].as_str().unwrap_or("").to_string();
        let eur = card["prices"]["eur"].as_str().map(String::from);
        out.push(ScryfallHit { name, mana_cost: cost, type_line: tl, eur });
    }
    Some(out)
}



pub fn spawn_worker(cfg: AgentConfig) -> Receiver<Out> {
    let (tx_out, rx_out) = std::sync::mpsc::channel::<Out>();
    let (tx_in, rx_in) = std::sync::mpsc::channel::<Vec<Msg>>();
    *TX.lock().unwrap() = Some(tx_in);

    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(120))
            .build();
        for history in rx_in {
            let mut messages = vec![json!({"role": "system", "content": SYSTEM_PROMPT})];
            for (role, content) in history {
                messages.push(json!({"role": role, "content": content}));
            }
            let payload = json!({
                "model": cfg.model,
                "messages": messages,
                "temperature": 0.4,
            });
            let result = agent
                .post(&format!("{}/chat/completions", cfg.base_url))
                .set("Authorization", &format!("Bearer {}", effective_key(&cfg)))
                .send_json(payload);
            match result {
                Ok(resp) => match resp.into_json::<Value>() {
                    Ok(body) => {
                        let content = body["choices"][0]["message"]["content"]
                            .as_str()
                            .unwrap_or("(leere Antwort)")
                            .to_string();
                        let _ = tx_out.send(Out::Reply(content));
                    }
                    Err(e) => {
                        let _ = tx_out.send(Out::Failed(format!("bad response: {e}")));
                    }
                },
                Err(ureq::Error::Status(code, resp)) => {
                    let body = resp.into_string().unwrap_or_default();
                    let msg = serde_json::from_str::<Value>(&body)
                        .ok()
                        .and_then(|v| v["error"]["message"].as_str().map(String::from))
                        .unwrap_or_else(|| body.chars().take(200).collect());
                    let _ = tx_out.send(Out::Failed(format!("HTTP {code}: {msg}")));
                }
                Err(e) => {
                    let _ = tx_out.send(Out::Failed(format!("{e}")));
                }
            }
        }
    });
    rx_out
}

/// Extrahiert {"add": [...], "remove": [...]} aus einer Agent-Antwort.
pub type DeckActions = (Vec<(String, i64)>, Vec<String>);

pub fn parse_actions(reply: &str) -> Option<DeckActions> {
    let start = reply.find("```")?;
    let rest = &reply[start..];
    let json_start = rest.find('{')?;
    let json_end = rest.rfind('}')?;
    if json_end <= json_start {
        return None;
    }
    let v: Value = serde_json::from_str(&rest[json_start..=json_end]).ok()?;
    let add = v["add"]
        .as_array()?
        .iter()
        .filter_map(|item| {
            let a = item.as_array()?;
            let name = a.first()?.as_str()?.to_string();
            let qty = a.get(1).and_then(|q| q.as_i64()).unwrap_or(1);
            Some((name, qty))
        })
        .collect();
    let remove = v["remove"]
        .as_array()?
        .iter()
        .filter_map(|r| r.as_str().map(std::string::ToString::to_string))
        .collect();
    Some((add, remove))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_action_block() {
        let r = "Analyse...\n```json\n{\"add\": [[\"Bolt\", 2]], \"remove\": [\"X\"]}\n```";
        let (add, remove) = parse_actions(r).unwrap();
        assert_eq!(add, vec![("Bolt".into(), 2)]);
        assert_eq!(remove, vec!["X".to_string()]);
    }

    #[test]
    fn no_actions_no_crash() {
        assert!(parse_actions("nur Text").is_none());
    }

    #[test]
    fn store_toggle_pin_note_roundtrip() {
        let path = format!(
            "{}/.mtg-tui-store-{}.json",
            std::env::temp_dir().display(),
            std::process::id()
        );
        std::env::set_var("MTG_TUI_STORE", &path);
        let mut store = Store {
            skills: vec![Skill { name: "context".into(), desc: "x", enabled: true }],
            ..Default::default()
        };
        assert!(store.toggle("context"));
        assert!(!store.skills[0].enabled);
        assert!(store.pin_toggle("Bolt"));
        assert_eq!(store.pinned, vec!["Bolt".to_string()]);
        assert!(!store.pin_toggle("Bolt")); // unpin
        store.add_note("Notiz".into());
        drop(store);
        let reloaded = Store::load();
        assert!(reloaded.notes.contains(&"Notiz".into()));
        assert!(reloaded.pinned.is_empty());
        assert!(!reloaded.skills[0].enabled);
        let _ = std::fs::remove_file(&path);
        std::env::remove_var("MTG_TUI_STORE");
    }
}

#[cfg(test)]
mod toggle_debug {
    use super::*;
    #[test]
    fn toggle_flips_enabled() {
        let path = format!("{}/tgl-{}.json", std::env::temp_dir().display(), std::process::id());
        std::env::set_var("MTG_TUI_STORE", &path);
        let mut s = Store {
            skills: vec![Skill { name: "context".into(), desc: "x", enabled: true }],
            ..Default::default()
        };
        let before = s.skills[0].enabled;
        assert!(s.toggle("context"));
        eprintln!("before={before} after={}", s.skills[0].enabled);
        assert_ne!(before, s.skills[0].enabled);
        std::env::remove_var("MTG_TUI_STORE");
        let _ = std::fs::remove_file(&path);
    }
}
