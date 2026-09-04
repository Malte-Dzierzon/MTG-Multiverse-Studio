//! App-State, Keybindings, Animation.

use crate::agent;
use crate::db::{category_of, Card, Db, Deck, DeckEntry};
use crate::theme::Theme;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Home,
    DeckView,
    Search,
    CardView,
}

/// Wo liegt der Tastenfokus: Hauptbereich oder Agent-Chat.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Focus {
    Main,
    Agent,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Input {
    None,
    Query,
    NewDeck,
    Import,
    Token,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SortMode {
    Alpha,
    Category,
}

pub const FORMATS: [&str; 8] = [
    "casual", "standard", "pioneer", "modern", "legacy", "vintage", "commander", "pauper",
];

#[derive(Clone, Debug)]
pub struct ChatMsg {
    pub user: bool,
    pub text: String,
}

pub struct App<'a> {
    pub db: &'a Db,
    pub theme: Theme,

    pub mode: Mode,
    pub focus: Focus,
    pub input: Input,
    pub query: String,
    pub status: String,

    pub decks: Vec<Deck>,
    pub deck_sel: usize,
    pub deck_anim: f32,
    pub slide: f32,
    pub home_entries: Option<Vec<DeckEntry>>,

    pub entries: Vec<DeckEntry>,
    pub entry_sel: usize,
    pub entry_anim: f32,
    pub active_deck: Option<Deck>,

    pub suggestions: Vec<Card>,
    pub sug_sel: usize,
    pub current_card: Option<Card>,

    pub spin_frame: usize,
    pub loading: bool,
    pub cover_loading: bool,
    pub requested: HashSet<String>,
    pub failed: HashSet<String>,

    pub cover_url: Option<String>,
    pub preview_url: Option<String>,

    // ── Deck-Ansicht ──
    pub sort: SortMode,

    // ── Format-Auswahl (neues Deck) ──
    pub pending_deck_name: String,
    pub fmt_open: bool,
    pub fmt_sel: usize,

    // ── Inline-Add im DeckView ──
    pub add_mode: bool,

    // ── Stat-Overlay ──
    pub stats_open: bool,

    // ── Deck-Werte (Cache fürs Dashboard) ──
    pub deck_values: Vec<f64>,

    // ── Agent: Deck-Picker ──
    pub deck_picker: bool,
    pub deck_pick_sel: usize,
    /// Wartet auf Scryfall-Ergebnisse für die nächste Agent-Runde.
    pub pending_search_followup: Option<(String, String)>,

    // ── Settings ──
    pub settings_open: bool,

    // ── Agent ──
    pub chat: Vec<ChatMsg>,
    pub agent_input: String,
    pub agent_busy: bool,
    pub store: agent::Store,
    pub cfg: agent::AgentConfig,
}

impl<'a> App<'a> {
    pub fn new(db: &'a Db, theme: Theme, cfg: agent::AgentConfig) -> Self {
        let decks = db.decks();
        let mut app = Self {
            db,
            theme,
            mode: Mode::Home,
            focus: Focus::Main,
            input: Input::None,
            query: String::new(),
            status: String::new(),
            decks,
            deck_sel: 0,
            deck_anim: 0.0,
            slide: 0.0,
            home_entries: None,
            entries: vec![],
            entry_sel: 0,
            entry_anim: 0.0,
            active_deck: None,
            suggestions: vec![],
            sug_sel: 0,
            current_card: None,
            spin_frame: 0,
            loading: false,
            cover_loading: false,
            requested: HashSet::new(),
            failed: HashSet::new(),
            sort: SortMode::Category,
            pending_deck_name: String::new(),
            fmt_open: false,
            fmt_sel: 0,
            add_mode: false,
            stats_open: false,
            deck_values: vec![],
            deck_picker: false,
            deck_pick_sel: 0,
            pending_search_followup: None,
            settings_open: false,
            cover_url: None,
            preview_url: None,
            chat: vec![],
            agent_input: String::new(),
            agent_busy: false,
            store: agent::Store::load(),
            cfg,
        };
        app.refresh_cover();
        app.chat.push(ChatMsg {
            user: false,
            text: "Frag mich etwas zu deinem Deck. Befehle: /help".into(),
        });
        app
    }

    // ── Daten ─────────────────────────────────────────────────────

    fn refresh_decks(&mut self) {
        self.decks = self.db.decks();
        self.deck_sel = self.deck_sel.min(self.decks.len().saturating_sub(1));
        self.deck_values = self
            .decks
            .iter()
            .map(|d| self.db.deck_cards(d.id).iter().map(|e| e.eur()).sum())
            .collect();
        self.refresh_cover();
    }

    /// Gefilterte Sicht auf die Decks (Home-'/'-Filter).
    pub fn visible_decks(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        self.decks
            .iter()
            .enumerate()
            .filter(|(_, d)| q.is_empty() || d.name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    fn apply_sort(&mut self) {
        match self.sort {
            SortMode::Alpha => self.entries.sort_by(|x, y| x.card.name.cmp(&y.card.name)),
            SortMode::Category => self
                .entries
                .sort_by(|x, y| x.category.cmp(&y.category).then(x.card.name.cmp(&y.card.name))),
        }
    }

    fn refresh_cover(&mut self) {
        match self.decks.get(self.deck_sel) {
            Some(d) => {
                self.home_entries = Some(self.db.deck_cards(d.id));
                self.cover_url = self.db.cover_card(d.id).and_then(|c| c.img_normal.clone());
            }
            None => {
                self.home_entries = None;
                self.cover_url = None;
            }
        }
    }

    fn open_selected_deck(&mut self) {
        let Some(deck) = self.decks.get(self.deck_sel).cloned() else { return };
        let mut entries = self.db.deck_cards(deck.id);
        std::mem::swap(&mut entries, &mut self.entries);
        self.active_deck = Some(deck);
        self.apply_sort();
        self.entry_sel = 0;
        self.entry_anim = 0.0;
        self.mode = Mode::DeckView;
        self.sync_preview_url();
    }

    fn sync_preview_url(&mut self) {
        self.preview_url = match self.mode {
            Mode::DeckView => self
                .entries
                .get(self.entry_sel)
                .and_then(|e| e.card.img_normal.clone()),
            Mode::CardView => self.current_card.as_ref().and_then(|c| c.img_normal.clone()),
            _ => None,
        };
    }

    // ── Animation ─────────────────────────────────────────────────

    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        let step = |anim: f32, target: f32| -> (f32, bool) {
            let next = anim + (target - anim) * (dt * 14.0).min(1.0);
            (next, (target - next).abs() > 0.01)
        };
        let (a, mv) = step(self.deck_anim, self.deck_sel as f32);
        self.deck_anim = a;
        moving |= mv;
        let (b, mv2) = step(self.entry_anim, self.entry_sel as f32);
        self.entry_anim = b;
        moving |= mv2;
        let (s, mv3) = step(self.slide, 1.0);
        self.slide = s;
        moving |= mv3;

        if self.loading || self.agent_busy {
            self.spin_frame = self.spin_frame.wrapping_add(1);
            moving = true;
        }
        moving
    }

    // ── Input-Dispatch ────────────────────────────────────────────

    /// true = quit.
    pub fn key(
        &mut self,
        code: crossterm::event::KeyCode,
        mods: crossterm::event::KeyModifiers,
    ) -> bool {
        use crossterm::event::{KeyCode as K, KeyModifiers};

        if mods.contains(KeyModifiers::CONTROL) && code == K::Char('d') {
            return true;
        }

        if self.focus == Focus::Agent {
            return self.key_agent(code);
        }

        match self.input {
            Input::Query => return self.key_query(code),
            Input::NewDeck => return self.key_new_deck(code),
            Input::Import => return self.key_import(code, mods),
            Input::None | Input::Token => {}
        }

        match code {
            K::Tab => self.focus = Focus::Agent,
            K::Char('q') if !self.stats_open => return true,
            K::Char('/') if self.mode == Mode::Home && !self.stats_open => {
                // Deck-Filter auf der Startseite
                self.input = Input::Query;
                self.query.clear();
            }
            K::Esc if self.stats_open => {
                self.stats_open = false;
            }
            _ => match self.mode {
                Mode::Home => self.key_home(code),
                Mode::DeckView => self.key_deck_view(code),
                Mode::CardView => match code {
                    K::Esc | K::Enter => {
                        self.mode = Mode::DeckView;
                        self.current_card = None;
                        self.sync_preview_url();
                    }
                    K::Char('a') => self.add_current_card(1),
                    K::Char('p') => {
                        self.pin_current();
                    }
                    _ => {}
                },
                Mode::Search => {}
            },
        }
        false
    }

    fn key_agent(&mut self, code: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode as K;
        if self.deck_picker {
            return self.key_deck_picker(code);
        }
        match code {
            K::Esc | K::Tab => self.focus = Focus::Main,
            K::Backspace => {
                self.agent_input.pop();
            }
            K::Enter => self.submit_chat(),
            _ => {}
        }
        false
    }

    fn key_deck_picker(&mut self, code: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode as K;
        match code {
            K::Esc => self.deck_picker = false,
            K::Down | K::Char('j') if !self.decks.is_empty() => {
                self.deck_pick_sel = (self.deck_pick_sel + 1).min(self.decks.len() - 1);
            }
            K::Up | K::Char('k') => self.deck_pick_sel = self.deck_pick_sel.saturating_sub(1),
            K::Enter => {
                if let Some(d) = self.decks.get(self.deck_pick_sel).cloned() {
                    self.status = format!("agent deck: {}", d.name);
                    let id = d.id;
                    self.active_deck = Some(d);
                    self.entries = self.db.deck_cards(id);
                    self.apply_sort();
                }
                self.deck_picker = false;
            }
            _ => {}
        }
        false
    }

    pub fn char_input(&mut self, ch: char) {
        if self.focus == Focus::Agent {
            self.agent_input.push(ch);
        } else {
            match self.input {
                Input::Query | Input::NewDeck | Input::Import => {
                    self.query.push(ch);
                    if self.input == Input::Query {
                        self.run_search();
                    }
                }
                Input::None | Input::Token => {}
            }
        }
    }

    pub fn paste(&mut self, s: &str) {
        if self.focus == Focus::Agent && s.len() < 8000 {
            self.agent_input.push_str(s);
        } else if self.input == Input::Import {
            self.query.push_str(s);
        }
    }

    fn submit_chat(&mut self) {
        let text = self.agent_input.trim().to_string();
        self.agent_input.clear();
        if text.is_empty() {
            return;
        }

        // Slash-Befehle
        if let Some(rest) = text.strip_prefix('/') {
            let (cmd, arg) = rest.split_once(' ').unwrap_or((rest, ""));
            match cmd {
                "note" => {
                    self.store.add_note(arg.to_string());
                    self.chat.push(ChatMsg { user: false, text: format!("Notiz gespeichert: {arg}") });
                }
                "pin" => {
                    if self.pin_current() {
                        // Meldung kommt aus pin_current
                    }
                }
                "toggle" | "skill" => {
                    if self.store.toggle(arg.trim()) {
                        self.chat.push(ChatMsg { user: false, text: format!("Skill '{arg}' umgeschaltet.") });
                    } else {
                        let names: Vec<_> = self.store.skills.iter().map(|k| k.name.clone()).collect();
                        self.chat.push(ChatMsg { user: false, text: format!("Unbekannter Skill. Verfügbar: {}", names.join(", ")) });
                    }
                }
                "deck" => {
                    if arg.trim().is_empty() {
                        self.deck_picker = true;
                        self.deck_pick_sel = 0;
                    } else if let Some(d) = self
                        .decks
                        .iter()
                        .find(|d| d.name.to_lowercase().contains(&arg.trim().to_lowercase()))
                        .cloned()
                    {
                        self.active_deck = Some(d.clone());
                        let id = d.id;
                        self.entries = self.db.deck_cards(id);
                        self.apply_sort();
                        self.chat.push(ChatMsg { user: false, text: format!("Aktives Deck: {}", d.name) });
                    } else {
                        self.chat.push(ChatMsg { user: false, text: format!("Deck '{arg}' nicht gefunden.") });
                    }
                }
                "clear" => {
                    self.chat.clear();
                    self.chat.push(ChatMsg { user: false, text: "Verlauf geleert.".into() });
                }
                "help" => {
                    self.chat.push(ChatMsg { user: false, text: "/deck [name] · /pin · /note <t> · /toggle <skill> · /skills · /clear".into() });
                }
                "skills" => {
                    for k in &self.store.skills {
                        let mark = if k.enabled { "●" } else { "○" };
                        self.chat.push(ChatMsg { user: false, text: format!("{mark} {} — {}", k.name, k.desc) });
                    }
                }
                other => {
                    self.chat.push(ChatMsg { user: false, text: format!("Unbekannter Befehl '/{other}' — /help") });
                }
            }
            return;
        }

        self.chat.push(ChatMsg { user: true, text: text.clone() });
        self.send_to_agent(text);
    }

    fn send_to_agent(&mut self, user_text: String) {
        if !self.cfg.ready() {
            self.chat.push(ChatMsg { user: false, text: "Kein API-Key (.env / MTG_LLM_API_KEY)".into() });
            return;
        }
        let mut context_blocks: Vec<String> = Vec::new();

        if self.store.skills.iter().any(|k| k.name == "context" && k.enabled) {
            let deck = self.active_deck.clone().or_else(|| self.decks.get(self.deck_sel).cloned());
            if let Some(d) = deck {
                let entries = self.db.deck_cards(d.id);
                context_blocks.push(agent_context_deck(&d.name, &d.fmt, &entries));
            }
        }
        if self.store.skills.iter().any(|k| k.name == "pins" && k.enabled) && !self.store.pinned.is_empty() {
            context_blocks.push(format!(
                "Angeheftete Karten:\n{}",
                self.store.pinned.join("\n")
            ));
        }
        if self.store.skills.iter().any(|k| k.name == "notes" && k.enabled) && !self.store.notes.is_empty() {
            context_blocks.push(format!(
                "Notizen des Nutzers:\n{}",
                self.store.notes.iter().map(|n| format!("- {n}")).collect::<Vec<_>>().join("\n")
            ));
        }

        let full = if context_blocks.is_empty() {
            user_text
        } else {
            format!("[KONTEXT]\n{}\n[/KONTEXT]\n\n{}", context_blocks.join("\n\n"), user_text)
        };

        let mut history: Vec<(String, String)> =
            self.chat.iter().filter(|m| !m.text.starts_with("Kein API") && m.user)
                .map(|m| ("user".into(), m.text.clone()))
                .collect();
        history.push(("user".into(), full));

        agent::ask(history);
        self.agent_busy = true;
    }

    /// Antwort des Workers einhängen (vom Event-Loop aufgerufen).
    pub fn on_agent_reply(&mut self, out: agent::Out) {
        self.agent_busy = false;
        match out {
            agent::Out::Reply(text) => {
                // Recherche-Loop: [[search: …]] → Scryfall → Folgefrage
                if let Some(rest) = text.strip_prefix("[[search:") {
                    if let Some((q, _)) = rest.rsplit_once("]]") {
                        let q = q.trim().to_string();
                        if self.store.skills.iter().any(|k| k.name == "scryfall" && k.enabled)
                            && !q.is_empty()
                        {
                            self.chat.push(ChatMsg {
                                user: false,
                                text: format!("recherche: {q} …"),
                            });
                            let followup = self.pending_search_followup.take();
                            agent::scryfall_search(q, followup.map(|c| vec![c]).unwrap_or_default());
                            self.agent_busy = true;
                            return;
                        }
                    }
                }
                let actions = agent::parse_actions(&text);
                let clean = text.split("```").next().unwrap_or(&text).trim_end();
                self.chat.push(ChatMsg { user: false, text: clean.to_string() });
                if let Some(actions) = actions {
                    self.apply_actions(actions);
                }
            }
            agent::Out::Failed(e) => {
                self.chat.push(ChatMsg { user: false, text: format!("[Fehler] {e}") });
            }
        }
    }

    /// Scryfall-Ergebnis eingehen lassen und dem Agent als Kontext nachreichen.
    pub fn on_scryfall_result(&mut self, query: String, hits: Vec<agent::ScryfallHit>) {
        let lines: Vec<String> = if hits.is_empty() {
            vec!["(keine Treffer)".into()]
        } else {
            hits.iter()
                .map(|h| {
                    format!(
                        "- {} {} | {} | €{}",
                        h.name,
                        h.mana_cost,
                        h.type_line,
                        h.eur.as_deref().unwrap_or("?")
                    )
                })
                .collect()
        };
        self.chat.push(ChatMsg {
            user: false,
            text: format!("scryfall '{query}':
{}", lines.join("
")),
        });
        self.agent_busy = true;
        let ctx = format!(
            "[TOOL-ERGEBNIS] Scryfall-Suche '{}':\n{}\n\nNutze diese Daten für deine Antwort.",
            query,
            lines.join("\n")
        );
        let mut history: Vec<(String, String)> = self
            .chat
            .iter()
            .filter(|m| m.user)
            .map(|m| ("user".into(), m.text.clone()))
            .collect();
        history.push(("user".into(), ctx));
        agent::ask(history);
    }

    fn apply_actions(&mut self, (add, remove): (Vec<(String, i64)>, Vec<String>)) {
        let Some(deck) = self.active_deck.clone() else {
            self.chat.push(ChatMsg { user: false, text: "(Änderungen übersprungen: kein Deck offen)".into() });
            return;
        };
        let mut applied: Vec<String> = Vec::new();
        for (name, qty) in add {
            if let Some(c) = self.db.card_by_name(&name) {
                self.db
                    .deck_add_card(deck.id, &c.id, qty, &category_of(&c.type_line));
                applied.push(format!("+{qty} {}", c.name));
            } else {
                applied.push(format!("?{name} nicht gefunden"));
            }
        }
        for name in remove {
            if let Some(c) = self.db.card_by_name(&name) {
                self.db.deck_remove_entry(deck.id, &c.id);
                applied.push(format!("-{}", c.name));
            } else {
                applied.push(format!("?{name} nicht gefunden"));
            }
        }
        if !applied.is_empty() {
            let id = deck.id;
            self.entries = self.db.deck_cards(id);
            self.refresh_decks();
            self.chat.push(ChatMsg { user: false, text: format!("Angewendet: {}", applied.join(", ")) });
        }
    }

    fn pin_current(&mut self) -> bool {
        let name = match (&self.mode, &self.current_card) {
            (_, Some(c)) => Some(c.name.clone()),
            (Mode::DeckView, None) => {
                self.entries.get(self.entry_sel).map(|e| e.card.name.clone())
            }
            _ => None,
        };
        let Some(name) = name else { return false };
        let pinned = self.store.pin_toggle(&name);
        self.status = if pinned {
            format!("📌 {name}")
        } else {
            format!("unpinned {name}")
        };
        self.chat.push(ChatMsg { user: false, text: self.status.clone() });
        pinned
    }

    // ── Main-Focus-Keyhandler ─────────────────────────────────────

    fn key_home(&mut self, code: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode as K;
        match code {
            K::Down | K::Char('j') if !self.decks.is_empty() => {
                self.deck_sel = (self.deck_sel + 1).min(self.decks.len() - 1);
                self.refresh_cover();
            }
            K::Up | K::Char('k') => {
                self.deck_sel = self.deck_sel.saturating_sub(1);
                self.refresh_cover();
            }
            K::Enter if !self.fmt_open => {
                let sel = self.visible_decks().get(self.deck_sel).copied();
                if let Some(i) = sel {
                    self.deck_sel = i;
                    self.refresh_cover();
                    self.open_selected_deck();
                }
            }
            K::Char('n') => {
                self.input = Input::NewDeck;
                self.query.clear();
            }
            K::Char('i') => {
                if let Some(d) = self.decks.get(self.deck_sel).cloned() {
                    let deck_id = d.id;
                    self.active_deck = Some(d);
                    self.entries = self.db.deck_cards(deck_id);
                    self.input = Input::Import;
                    self.query.clear();
                    self.mode = Mode::DeckView;
                    self.status = "paste deck list · enter newlines · ctrl+s import · esc abort".into();
                }
            }
            K::Char('x') => {
                if let Some(d) = self.decks.get(self.deck_sel) {
                    let id = d.id;
                    self.db.deck_delete(id);
                    self.deck_sel = self.deck_sel.saturating_sub(1);
                    self.refresh_decks();
                    self.deck_anim = self.deck_sel as f32;
                }
            }
            K::Char('e') => {
                if let Some(d) = self.decks.get(self.deck_sel).cloned() {
                    let text = self.db.deck_export_text(d.id, &d.name);
                    std::fs::create_dir_all("decks").ok();
                    let stamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|s| s.as_secs())
                        .unwrap_or(0);
                    let path = format!("decks/{}-{stamp}.txt", slugify(&d.name));
                    match std::fs::write(&path, text) {
                        Ok(_) => self.status = format!("exported → {path}"),
                        Err(e) => self.status = format!("export failed: {e}"),
                    }
                }
            }
            _ => {}
        }
    }

    fn key_deck_view(&mut self, code: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode as K;
        match code {
            K::Down | K::Char('j') if !self.entries.is_empty() => {
                self.entry_sel = (self.entry_sel + 1).min(self.entries.len() - 1);
                self.sync_preview_url();
            }
            K::Up | K::Char('k') => {
                self.entry_sel = self.entry_sel.saturating_sub(1);
                self.sync_preview_url();
            }
            K::Esc => {
                self.mode = Mode::Home;
                self.preview_url = None;
                self.refresh_cover();
            }
            K::Char('d') => {
                if let (Some(deck), Some(e)) =
                    (self.active_deck.clone(), self.entries.get(self.entry_sel))
                {
                    let card_id = e.card.id.clone();
                    self.db.deck_remove_entry(deck.id, &card_id);
                    self.entries = self.db.deck_cards(deck.id);
                    self.entry_sel = self.entry_sel.min(self.entries.len().saturating_sub(1));
                    self.refresh_decks();
                    self.sync_preview_url();
                }
            }
            K::Enter => {
                if let Some(e) = self.entries.get(self.entry_sel) {
                    self.current_card = Some(e.card.clone());
                    self.mode = Mode::CardView;
                    self.sync_preview_url();
                }
            }
            K::Char('a') => {
                // Inline-Add: Suche öffnen, Enter fügt Karte +1 hinzu
                self.add_mode = true;
                self.input = Input::Query;
                self.mode = Mode::Search;
                self.query.clear();
                self.suggestions.clear();
            }
            K::Char('t') => {
                self.sort = match self.sort {
                    SortMode::Alpha => SortMode::Category,
                    SortMode::Category => SortMode::Alpha,
                };
                self.apply_sort();
                self.entry_sel = 0;
                self.entry_anim = 0.0;
                let name = self.entries.get(self.entry_sel).map(|e| e.card.name.clone());
                self.preview_url = name.and_then(|n| self.db.card_by_name(&n)).and_then(|c| c.img_normal.clone());
            }
            K::Char('s') => self.stats_open = true,
            K::Char('p') => {
                self.pin_current();
            }
            _ => {}
        }
    }

    fn key_import(
        &mut self,
        code: crossterm::event::KeyCode,
        mods: crossterm::event::KeyModifiers,
    ) -> bool {
        use crossterm::event::{KeyCode as K, KeyModifiers};
        match code {
            K::Esc => {
                self.input = Input::None;
                self.query.clear();
                self.status.clear();
            }
            K::Char('s') if mods.contains(KeyModifiers::CONTROL) => {
                let text = std::mem::take(&mut self.query);
                self.finish_import(&text);
            }
            K::Enter => self.query.push('\n'),
            K::Tab => self.query.push('\t'),
            K::Backspace => {
                self.query.pop();
            }
            _ => {}
        }
        false
    }

    fn finish_import(&mut self, text: &str) {
        if let Some(deck) = &self.active_deck {
            let (n, missing) = self.db.deck_import_text(deck.id, text);
            self.status = if missing.is_empty() {
                format!("imported {n} lines")
            } else {
                format!(
                    "imported {n}, missing: {}",
                    missing.iter().take(4).cloned().collect::<Vec<_>>().join(", ")
                )
            };
            let id = deck.id;
            self.entries = self.db.deck_cards(id);
            self.refresh_decks();
        }
        self.input = Input::None;
        self.mode = Mode::DeckView;
    }

    fn key_query(&mut self, code: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode as K;
        match code {
            K::Esc | K::Left => {
                self.input = Input::None;
                self.query.clear();
                self.suggestions.clear();
                if self.add_mode {
                    self.add_mode = false;
                    self.mode = Mode::DeckView;
                } else {
                    self.mode = Mode::Home;
                }
            }
            K::Enter => {
                if self.add_mode {
                    let q = self.query.trim().to_string();
                    if let Some(c) = self
                        .suggestions
                        .first()
                        .cloned()
                        .or_else(|| self.db.card_by_name(&q))
                    {
                        if let Some(deck) = &self.active_deck {
                            let did = deck.id;
                            self.db.deck_add_card(did, &c.id, 1, &category_of(&c.type_line));
                            self.status = format!("+1 {}", c.name);
                            self.entries = self.db.deck_cards(did);
                            self.apply_sort();
                            self.refresh_decks();
                        }

                        self.query.clear();
                        self.suggestions.clear();
                    }
                    return false;
                }
                let pick = self
                    .suggestions
                    .get(self.sug_sel)
                    .cloned()
                    .or_else(|| self.suggestions.first().cloned())
                    .or_else(|| self.db.card_by_name(self.query.trim()));
                if let Some(c) = pick {
                    self.current_card = Some(c);
                    self.mode = Mode::CardView;
                    self.add_mode = false;
                    self.input = Input::None;
                    self.query.clear();
                    self.suggestions.clear();
                    self.sync_preview_url();
                }
            }
            K::Backspace => {
                self.query.pop();
                self.run_search();
            }
            K::Down => {
                if self.sug_sel + 1 < self.suggestions.len() {
                    self.sug_sel += 1;
                }
            }
            K::Up => self.sug_sel = self.sug_sel.saturating_sub(1),
            K::Char(ch) => {
                self.query.push(ch);
                self.run_search();
            }
            _ => {}
        }
        false
    }

    fn run_search(&mut self) {
        let q = self.query.trim().to_string();
        self.sug_sel = 0;
        self.suggestions = if q.len() >= 2 { self.db.search(&q, 12) } else { vec![] };
    }

    fn key_new_deck(&mut self, code: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode as K;
        if self.fmt_open {
            return self.key_format_picker(code);
        }
        match code {
            K::Esc => {
                self.input = Input::None;
                self.query.clear();
            }
            K::Enter => {
                let name = self.query.trim().to_string();
                if !name.is_empty() {
                    self.pending_deck_name = name;
                    self.fmt_open = true;
                    self.fmt_sel = 0;
                }
            }
            K::Backspace => {
                self.query.pop();
            }
            K::Char(ch) => self.query.push(ch),
            _ => {}
        }
        false
    }

    fn key_format_picker(&mut self, code: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode as K;
        match code {
            K::Esc => {
                self.fmt_open = false;
                self.pending_deck_name.clear();
                self.input = Input::None;
                self.query.clear();
            }
            K::Down | K::Char('j') => {
                self.fmt_sel = (self.fmt_sel + 1).min(FORMATS.len() - 1);
            }
            K::Up | K::Char('k') => self.fmt_sel = self.fmt_sel.saturating_sub(1),
            K::Enter => {
                let fmt = FORMATS[self.fmt_sel];
                let deck = self.db.deck_create_fmt(&self.pending_deck_name, fmt);
                self.fmt_open = false;
                self.pending_deck_name.clear();
                self.input = Input::None;
                self.query.clear();
                self.refresh_decks();
                if let Some(i) = self.decks.iter().position(|d| d.id == deck.id) {
                    self.deck_sel = i;
                }
                self.open_selected_deck();
            }
            _ => {}
        }
        false
    }

    fn add_current_card(&mut self, qty: i64) {
        let Some(card) = self.current_card.clone() else { return };
        let Some(deck) = self.active_deck.clone() else {
            self.status = "no deck open".into();
            return;
        };
        self.db
            .deck_add_card(deck.id, &card.id, qty, &category_of(&card.type_line));
        self.status = format!("+{qty} {} → {}", card.name, deck.name);
        if self.mode == Mode::DeckView {
            self.entries = self.db.deck_cards(deck.id);
            self.refresh_decks();
        }
    }
}

fn agent_context_deck(name: &str, fmt: &str, entries: &[DeckEntry]) -> String {
    let mut lines = vec![format!("Deck: {name} ({fmt})")];
    for e in entries {
        let eur = e.card.prices.get("eur").and_then(|v| v.as_str());
        let price = eur.map(|p| format!(" | €{p}")).unwrap_or_default();
        lines.push(format!(
            "{}x {} – {} | {} | {}{}",
            e.qty, e.card.name, e.card.mana_cost, e.card.type_line, e.card.set_code, price
        ));
    }
    lines.push(format!("Gesamt: {}", entries.iter().map(|e| e.qty).sum::<i64>()));
    lines.join("\n")
}

fn slugify(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    s.trim_matches('_').to_string()
}


#[cfg(test)]
mod redesign_tests {
    use super::*;
    use crate::db::Db;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn app() -> App<'static> {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("mtgr-{n}.db"));
        let sp = std::env::temp_dir().join(format!("mtgr-{n}.json"));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&sp);
        std::fs::copy("assets/mtg_multiverse_studio.db", &path).unwrap();
        std::env::set_var("MTG_TUI_STORE", sp.display().to_string());
        let db = Box::leak(Box::new(Db::open(path.to_str().unwrap()).unwrap()));
        App::new(db, Theme::default(), agent::AgentConfig::from_env())
    }

    #[test]
    fn new_deck_with_format_picker() {
        let mut a = app();
        a.key(KeyCode::Char('n'), KeyModifiers::NONE);
        "Cmdr".chars().for_each(|c| a.char_input(c));
        a.key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(a.fmt_open);
        // zu "commander" navigieren (Index 6)
        for _ in 0..6 {
            a.key(KeyCode::Down, KeyModifiers::NONE);
        }
        assert_eq!(FORMATS[a.fmt_sel], "commander");
        a.key(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(a.mode, Mode::DeckView);
        assert_eq!(a.active_deck.as_ref().unwrap().fmt, "commander");
        assert_eq!(
            a.db.decks()[0].name, "Cmdr",
            "neuestes Deck zuerst"
        );
    }

    #[test]
    fn deck_filter_on_home() {
        let mut a = app();
        // Eindeutiger Name — die Nutzer-DB kann gleichnamige Testdecks enthalten
        let unique = format!("Fltr{}", std::process::id());
        let _ = a.db.deck_create_fmt(&unique, "modern");
        a.refresh_decks();
        press_filter(&mut a, &unique.to_lowercase());
        assert_eq!(a.visible_decks().len(), 1);
        press_filter(&mut a, "");
        assert_eq!(a.visible_decks().len(), a.decks.len());
    }

    fn press_filter(a: &mut App<'_>, s: &str) {
        a.input = Input::Query;
        a.query.clear();
        for c in s.chars() {
            a.char_input(c);
        }
    }

    #[test]
    fn sort_toggle_and_inline_add() {
        let mut a = app();
        // Eigenes, kontrolliertes Deck
        let deck = a.db.deck_create_fmt("SortT", "casual");
        a.active_deck = Some(deck.clone());
        a.mode = Mode::DeckView;

        let bolt = a.db.card_by_name("Lightning Bolt").unwrap();
        let boseiju = a.db.card_by_name("Boseiju, Who Endures").unwrap();
        let did = deck.id;
        a.db.deck_add_card(did, &bolt.id, 4, &category_of(&bolt.type_line));
        a.db.deck_add_card(did, &boseiju.id, 1, &category_of(&boseiju.type_line));
        a.entries = a.db.deck_cards(did);

        // Alpha: Boseiju vor Lightning Bolt
        a.sort = SortMode::Alpha;
        a.apply_sort();
        assert_eq!(a.entries[0].card.name, "Boseiju, Who Endures");
        // Category: Creatures vor Instants → Bolt zuerst
        a.sort = SortMode::Category;
        a.apply_sort();
        assert_eq!(a.entries[0].card.name, "Lightning Bolt");

        // Inline-Add über Search+add_mode
        a.add_mode = true;
        a.input = Input::Query;
        a.mode = Mode::Search;
        "lightning bolt".chars().for_each(|c| a.char_input(c));
        a.key(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(a.status, "+1 Lightning Bolt");
        assert_eq!(a.entries.iter().find(|e| e.card.name == "Lightning Bolt").unwrap().qty, 5);
    }

    #[test]
    fn scryfall_loop_reaches_agent() {
        let mut a = app();
        a.store.toggle("scryfall");
        // Simulierte Agent-Antwort mit Recherche-Marker
        a.on_agent_reply(agent::Out::Reply("[[search: lightning bolt]]".into()));
        assert!(a.agent_busy, "wartet auf Scryfall-Ergebnis");
        a.on_scryfall_result(
            "lightning bolt".into(),
            vec![agent::ScryfallHit {
                name: "Lightning Bolt".into(),
                mana_cost: "{R}".into(),
                type_line: "Instant".into(),
                eur: Some("1.20".into()),
            }],
        );
        assert!(a.agent_busy, "wartet auf finale Agent-Antwort");
        assert!(
            a.chat.iter().any(|m| m.text.contains("scryfall 'lightning bolt'")),
            "Tool-Ergebnis im Verlauf"
        );
    }

    #[test]
    fn slash_commands_work() {
        let mut a = app();
        a.focus = Focus::Agent;
        "/help".chars().for_each(|c| a.char_input(c));
        a.key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(a.chat.iter().any(|m| m.text.contains("/pin")));
        "/toggle context".chars().for_each(|c| a.char_input(c));
        assert_eq!(a.agent_input, "/toggle context");
        a.key(KeyCode::Enter, KeyModifiers::NONE);
        let ctx_state = a
            .store
            .skills
            .iter()
            .find(|k| k.name == "context")
            .unwrap()
            .enabled;
        let last: Vec<_> = a.chat.iter().map(|m| m.text.clone()).collect();
        assert!(!ctx_state, "chat: {last:?} | skills: {:?}", a.store.skills);
        "/note mana fixen".chars().for_each(|c| a.char_input(c));
        a.key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(a.store.notes.contains(&"mana fixen".to_string()));
    }

    #[test]
    fn tab_focuses_agent_panel() {
        let mut a = app();
        assert_eq!(a.focus, Focus::Main);
        a.key(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(a.focus, Focus::Agent);
        "hallo".chars().for_each(|c| a.char_input(c));
        assert_eq!(a.agent_input, "hallo");
        a.key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(a.focus, Focus::Main);
        assert_eq!(a.agent_input, "hallo"); // Text bleibt erhalten
    }

}
