# MTG Multiverse Studio — Rust Backend Architekturplan

> **Status:** ✅ Implementiert — siehe `src-tauri/src/` für den fertigen Code  
> **Datum:** 2026-07-01  
> **Version:** v0.1 (Architektur-Referenz — Code ist gebaut, Plan dient als Dokumentation)

---

## 1. Projektzustand & -ziel

### Aktueller Zustand
- ✅ Grundlegende Ordnerstruktur existiert (`src-tauri/src/` mit `db/`, `scryfall/`, `ki/`, `utils/`)
- ✅ Config-Dateien angelegt (`Cargo.toml`, `tauri.conf.json`, `package.json`)
- ✅ Dokumentationsgrundlage vorhanden (Roadmap, Umsetzungsmöglichkeiten)
- ✅ **Alle `.rs` Dateien sind implementiert und kompilieren fehlerfrei**
- ✅ **Phase 1 (Backend) abgeschlossen** — siehe `ROADMAP.md` für Details

### Ziel dieses Plans
Ein durchdachtes, skalierbares Rust-Backend zu entwerfen, das:
1. Scryfall-API-Daten effizient abruft und in SQLite speichert
2. Über Tauri Commands dem React-Frontend Daten bereitstellt  
3. Später On-Device LLM Inferenz unterstützt (KI-Assistent)

---

## 2. Architekturübersicht

### 2.1 Systemarchitektur (Tauri v2)

```
┌─────────────────────────────────────────────┐
│                   Frontend                   │
│              React + TypeScript              │
│                                              │
│   invoke('search_cards', { query: "..." })  │
│   invoke('get_card',    { id:    "..." })   │
│   invoke('create_deck', { name:  "..." })   │
└──────────────────┬──────────────────────────┘
                   │ Tauri Bridge (IPC)
                   ▼
┌─────────────────────────────────────────────┐
│               Rust Backend                  │
│                                              │
│  ┌────────────┐  ┌────────────────────────┐ │
│  │ Commands   │  │     Services           │ │
│  │ (public API)│  │                        │ │
│  │            │  │  ┌──────────────────┐  │ │
│  ├─get_card   │◄─┤  │ CardService      │  │ │
│  ├─search_    │  │  ├──────────────────┤  │ │
│  ├─create_... │  │  │ ScryfallService  │  │ │
│  └────────────┘  │  ├──────────────────┤  │ │
│                  │  │ DeckService      │  │ │
│                  │  └──────────────────┘  │ │
│                  └────────────────────────┘ │
│                                              │
│  ┌────────────┐  ┌────────────────────────┐ │
│  │ Database   │  │     Models             │ │
│  │ (SQLite)   │  │                        │ │
│  │            │  │  struct Card { ... }   │ │
│  ├─cards      │  │  struct Deck { ... }   │ │
│  ├─collection │  │  struct LoreEntry{...} │ │
│  ├─decks      │  └────────────────────────┘ │
│  └────────────┘                              │
└──────────────────────────────────────────────┘
                   │
         ┌─────────┴──────────┐
         ▼                    ▼
   Scryfall API          SQLite DB
   (HTTP)               (lokal)
```

### 2.2 Module-Struktur (src-tauri/src/)

```
src/
├── main.rs                  # Entry Point, Tauri Setup, Command-Registrierung
├── commands.rs              # ❗PUBLIC API — alle Tauri Commands hier definiert
│
├── db/                      # Datenbank-Layer (Repository Pattern)
│   ├── mod.rs               # pub use re-exports
│   ├── schema.rs            # SQL CREATE TABLE Statements + Migrations
│   ├── connection.rs        # Connection Pool Management
│   ├── card_repo.rs         # CRUD für cards Tabelle
│   ├── deck_repo.rs         # CRUD für decks/deck_cards Tabellen
│   ├── collection_repo.rs   # CRUD für collection Tabelle
│   └── lore_repo.rs         # CRUD für lore_entries
│
├── scryfall/                # Scryfall API Integration
│   ├── mod.rs
│   ├── client.rs            # reqwest::Client Wrapper mit Rate-Limiting
│   ├── models.rs             # Rust structs: Card, Set, Price etc. (Serde)
│   └── cache.rs              # Lokales Caching + Cache-Invalidation
│
├── services/                # Business Logic Layer
│   ├── mod.rs
│   ├── card_service.rs       # Koordiniert DB + Scryfall für Kartendaten
│   ├── deck_service.rs       # Deck-Erstellung, Analyse, Wahrscheinlichkeiten
│   └── lore_service.rs        # Markdown-Parsing, Asset-Management
│
├── models/                  # Shared Data Models (Frontend-Kommunikation)
│   ├── mod.rs
│   ├── card.rs              # CardResponse (was an Frontend geht — serielisiert!)
│   ├── deck.rs              
│   └── collection.rs        
│
└── utils/                   # Hilfsfunktionen
    ├── mod.rs
    └── error.rs             # Custom Error Types + Result Wrapper
```

---

## 3. Datenbank-Schema (SQLite)

### 3.1 Tabellen-Design

#### `cards` — Kartendatenbank (wird von Scryfall befüllt)

| Spalte            | Typ        | Beschreibung                          |
|------------------|-----------|---------------------------------------|
| id                | TEXT PK    | Scryfall UUID (`bd8fa327-...`)        |
| oracle_id         | TEXT       | Oracle Text ID (für Drucker-Deduplizierung) |
| name              | TEXT NOT NULL | Kartentitel                        |
| mana_cost         | TEXT       | z.B. `"{1}{W}"`                       |
| cmc               | REAL       | Converted Mana Cost                   |
| type_line         | TEXT       | z.B. `"Creature — Wizard"`            |
| oracle_text       | TEXT        | Kartentext                            |
| colors            | TEXT        | JSON-Array `["W","U"]`                |
| color_identity    | TEXT        | JSON-Array                             |
| keywords          | TEXT        | JSON-Array z.B. `["Flying", "Trample"]` |
| rarity            | TEXT        | common/uncommon/rare/mythic/legendary   |
| set_id            | TEXT       | FK zu `sets.id`                        |
| image_uris        | TEXT        | JSON mit small/large/png URIs          |
| artist            | TEXT                          |
| legalities         | TEXT      | JSON (alle Format-Legalitys)           |
| prices             | TEXT      | JSON (USD, EUR etc.)                   |

**Indizes:** `name`, `oracle_id`, `(set_id, rarity)`

#### `collection` — Benutzer-Sammlung

| Spalte            | Typ        | Beschreibung                          |
|------------------|-----------|---------------------------------------|
| id                | INTEGER PK AUTOINCREMENT |
| card_id           | TEXT FK → cards.id (NOT NULL) |
| quantity          | INTEGER DEFAULT 1         |
| condition         | TEXT DEFAULT 'nm'         | nm/ex/lp/mp/po                         |
| notes             | TEXT                       | Benutzer-Notizen                        |
| added_at          | DATETIME DEFAULT CURRENT_TIMESTAMP |

**Indizes:** `card_id` (unique — pro Karte nur ein Eintrag)

#### `decks` — Deck-Metadaten

| Spalte            | Typ        | Beschreibung                          |
|------------------|-----------|---------------------------------------|
| id                | INTEGER PK AUTOINCREMENT |
| name              | TEXT NOT NULL                      |
| format            | TEXT                               | standard/modern/commander/legacy/etc.   |
| description       | TEXT                                |
| created_at        | DATETIME DEFAULT CURRENT_TIMESTAMP  |
| updated_at        | DATETIME                            |

#### `deck_cards` — Deck ↔ Card Relation (Many-to-Many)

| Spalte            | Typ        | Beschreibung                          |
|------------------|-----------|---------------------------------------|
| id                | INTEGER PK AUTOINCREMENT |
| deck_id           | INTEGER FK → decks.id (NOT NULL)   |
| card_id           | TEXT FK → cards.id (NOT NULL)       |
| quantity          | INTEGER DEFAULT 1                   |
| position          | INTEGER                            | Hauptkarte? Sideboard?                  |

**Indizes:** `(deck_id, card_id)` unique pair

#### `sets` — MTG Sets (aus Scryfall)

| Spalte            | Typ        | Beschreibung                          |
|------------------|-----------|---------------------------------------|
| id                | TEXT PK    | Scryfall Set UUID                     |
| name              | TEXT NOT NULL                        |
| set_type          | TEXT                                   | expansion/masters/supplemental...       |
| released          | DATE                                    |
| collector_docket_number | TEXT                            |

#### `lore_entries` — Lore- & Story-Einträge

| Spalte            | Typ        | Beschreibung                          |
|------------------|-----------|---------------------------------------|
| id                | INTEGER PK AUTOINCREMENT |
| title             | TEXT NOT NULL                         |
| lore_type         | TEXT                                  | story/artwork/character/plane           |
| content_path      | TEXT                                   | Pfad zur Markdown-Datei in `assets/stories/` |
| metadata          | TEXT                                   | YAML Frontmatter als JSON               |
| related_cards     | TEXT                                   | JSON Array von card_ids                 |

### 3.2 SQL-Migration (001_initial.sql)

```sql
-- Migration: Initial Schema
-- Created: 2026-07-01

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    set_type TEXT,
    released DATE,
    collector_number_prefix TEXT
);

CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    oracle_id TEXT UNIQUE,
    name TEXT NOT NULL,
    mana_cost TEXT,
    cmc REAL,
    type_line TEXT,
    oracle_text TEXT,
    colors TEXT DEFAULT '[]',
    color_identity TEXT DEFAULT '[]',
    keywords TEXT DEFAULT '[]',
    rarity TEXT,
    set_id TEXT REFERENCES sets(id),
    image_uris TEXT,
    artist TEXT,
    legalities TEXT DEFAULT '{}',
    prices TEXT DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_cards_name ON cards(name);
CREATE INDEX IF NOT EXISTS idx_cards_oracle_id ON cards(oracle_id);
CREATE INDEX IF NOT EXISTS idx_cards_set_rarity ON cards(set_id, rarity);

CREATE TABLE IF NOT EXISTS collection (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id TEXT NOT NULL UNIQUE REFERENCES cards(id),
    quantity INTEGER DEFAULT 1,
    condition TEXT DEFAULT 'nm',
    notes TEXT,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_collection_card ON collection(card_id);

CREATE TABLE IF NOT EXISTS decks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    format TEXT,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);

CREATE TABLE IF NOT EXISTS deck_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    card_id TEXT NOT NULL REFERENCES cards(id),
    quantity INTEGER DEFAULT 1,
    position INTEGER,
    UNIQUE(deck_id, card_id)
);

CREATE INDEX IF NOT EXISTS idx_deck_cards_lookup ON deck_cards(deck_id, card_id);

CREATE TABLE IF NOT EXISTS lore_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    lore_type TEXT,
    content_path TEXT,
    metadata TEXT DEFAULT '{}',
    related_cards TEXT DEFAULT '[]'
);
```

---

## 4. Scryfall API Integration — Strategie

### 4.1 Datenstrategie

**Problem:** Scryfall hat Tausende Karten mit jeweils riesigen JSON-Responses (~5KB+ pro Karte).  
Wir können nicht alles auf einmal laden und dürfen nicht bei jedem Frontend-Klick die API spammen.

**Lösung: Dreistufiges Caching**

```
Frontend Request
    │
    ▼
┌───────────────┐
│ 1. SQLite DB?  │ ◄── Erste Wahl! Schnell, lokal.
└───────┬────────┘
        │ Nicht gefunden
        ▼
┌───────────────┐
│ 2. LRU Cache   │ ◄── In-Memory Cache (Arc<Mutex<LruCache>>)
│    (recent)    │     für die letzten ~50 abgerufenen Karten
└───────┬────────┘
        │ Nicht gefunden, Cache leer/tot
        ▼
┌───────────────┐
│ 3. Scryfall   │ ◄── Echte API-Abfrage (Rate-Limited!)
│    API Call    │     → Ergebnis wird in SQLite + LRC gespeichert
└───────────────┘
```

### 4.2 Rate Limiting & Concurrent Requests

```rust
// client.rs — ScryfallClient mit eingebautem Rate Limiting
use reqwest::Client;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

pub struct ScryfallClient {
    http: Client,
    semaphore: Arc<Semaphore>, // Max 5 parallele Requests (Scryfall Limit)
    cache: Arc<Mutex<LruCache<String, Card>>>, // In-Memory LRU Cache
}

impl ScryfallClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            semaphore: Arc::new(Semaphore::new(5)),
            cache: Arc::new(Mutex::new(LruCache::<String, Card>::new(64))),
        }
    }

    pub async fn get_card(&self, id: &str) -> Result<Card> { ... }
    pub async fn search_cards(&self, query: &str) -> Result<Vec<Card>> { ... }
}
```

### 4.3 Models — Scryfall JSON → Rust Structs (Serde Deserialization)

Das ist das Herzstück! Die Scryfall API liefert sehr detailliertes JSON. Wir müssen es effizient in unsere SQLite-Struktur transformieren.

**Beispiel (aus der Recherche):** Black Lotus Response hat ~60+ Felder, aber wir speichern nur die relevanten:

```rust
// scryfall/models.rs — Was von der API kommt
#[derive(Debug, Clone, Deserialize)]
pub struct ScryfallCard {
    pub id: String,                    // "bd8fa327-dd41-..."
    #[serde(rename = "oracle_id")]
    pub oracle_id: Option<String>,     // Drucker-Deduplizierung
    pub name: String,                  // "Black Lotus"
    #[serde(rename = "mana_cost")]
    pub mana_cost: Option<String>,      // "{0}"
    #[serde(rename = "cmc")]
    pub cmc: Option<f64>,               // 0.0
    #[serde(rename = "type_line")]
    pub type_line: String,              // "Artifact"
    #[serde(rename = "oracle_text")]
    pub oracle_text: Option<String>,   // "{T}, Sacrifice..."
    
    pub colors: Vec<Color>,             // ["W","U","B","R","G"]
    #[serde(rename = "color_identity")]
    pub color_identity: Vec<Color>,
    pub keywords: Vec<String>,
    
    // Images (für Frontend!)
    #[serde(rename = "image_uris")]
    pub image_uris: Option<ImageUris>,  // { small, normal, large, png }
    
    pub rarity: String,                 // "common", "rare" etc.
    pub set: SetInfo,                   // Embeddeter Set-Block
    
    // ... viele weitere Felder (legalities, prices, etc.)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageUris {
    pub small: String,
    #[serde(rename = "normal")]
    pub normal_size: String,
    pub large: String,
    pub png: Option<String>,
}

// models/card.rs — Was das Frontend bekommt (vereinfacht!)
#[derive(Debug, Clone, Serialize)]
pub struct CardResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mana_cost: Option<String>,
    pub cmc: f64,
    pub type_line: String,
    #[serde(rename = "oracle_text")]
    pub card_text: String,
    pub colors: Vec<String>,
    pub rarity: String,
    pub set_name: String,
    pub artist: Option<String>,
    
    // Frontend braucht Bilder!
    pub image_url_small: Option<String>,  // für Thumbnails
    pub image_url_large: Option<String>,  // für Detail-Ansicht
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prices: Option<CardPricesResponse>,
}

// card_repo.rs — Was in SQLite gespeichert wird (noch vereinfachter)
#[derive(Debug, Clone)]
pub struct CardDb {
    pub id: String,         // Scryfall UUID
    pub oracle_id: Option<String>,
    pub name: String,
    pub mana_cost: Option<String>,
    pub cmc: f64,
    pub type_line: String,
    pub oracle_text: String,
    pub colors: Vec<Color>,     → gespeichert als JSON-String "[]"
    pub rarity: String,
    pub set_id: String,         // FK zur sets Tabelle
    pub image_uris_json: String, → gespeichert als JSON-String
}
```

---

## 5. Tauri Commands — Die Public API

### 5.1 Command-Definition (commands.rs)

Alle Commands leben in einer einzigen Datei und werden in `main.rs` registriert:

```rust
// commands.rs
use tauri::command;
use serde::{Serialize, Deserialize};

// ─── INPUT TYPES (vom Frontend kommen) ──────────────

#[derive(Debug, Deserialize)]
pub struct SearchCardsArgs {
    pub query: String,
}

#[derive(Debug, Deserialize)] 
pub struct GetCardArgs {
    pub id: String,  // Scryfall card ID
}

// ─── OUTPUT TYPES (zum Frontend) ─────────────────────

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub cards: Vec<CardResponse>,
    pub total: usize,
    pub from_cache: bool,
}

// ─── COMMANDS ─────────────────────────────────────────

#[command]
pub async fn search_cards(
    app_handle: tauri::AppHandle,  // Zugriff auf App-State!
    args: SearchCardsArgs,
) -> Result<SearchResult, String> {
    let state = app_handle.state::<AppState>();
    
    CardService::search(&state.db_pool, &state.scryfall_client, &args.query).await
}

#[command]
pub async fn get_card(
    app_handle: tauri::AppHandle,
    args: GetCardArgs,
) -> Result<CardResponse, String> { ... }

#[command]
pub async fn add_to_collection(
    _app_handle: tauri::AppHandle,
    args: AddToCollectionArgs,
) -> Result<CollectionItem, String> { ... }

#[command] 
pub async fn create_deck(
    app_handle: tauri::AppHandle,
    args: CreateDeckArgs,
) -> Result<CreatedDeck, String> { ... }

// Mehr Commands...
```

### 5.2 main.rs — Setup & Registrierung

```rust
// main.rs
mod db;
mod scryfall;  
mod commands;
mod models;
mod utils;

use tauri::Manager;
use std::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    pub db: rusqlite::Connection,           // SQLite Connection (thread-safe via Mutex)
    pub scryfall_client: Arc<ScryfallClient>, // HTTP Client mit Rate-Limiting
}

fn main() {
    tauri::Builder::default()
        .setup(|_api, _handle, _paths| {
            // 1. SQLite DB erstellen/öffnen + Migrations ausführen
            let db = db::connection::init_db()?;
            
            // 2. Scryfall Client erstellen  
            let scryfall_client = Arc::new(ScryfallClient::new());
            
            // 3. App-State initialisieren und speichern
            _handle.manage(AppState { 
                db, 
                scryfall_client: scryfall.clone() 
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_cards,
            commands::get_card,
            commands::add_to_collection,
            commands::create_deck,
            commands::load_lore_entries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 6. Performance-Strategie

### 6.1 Große Datenmengen effizient verarbeiten

MTG hat **25,000+ Karten** (Standalone Prints, nicht Oracle Unique). Das ist viel!

| Problem | Lösung |
|---------|--------|
| Alle Karten auf einmal laden? | ❌ NEIN — Lazy Loading: Nur nach Suche/Name laden |
| Frontend wird bei 1000 Karten langsam? | Pagination + Virtual Lists (Frontend-Seite) |
| Datenbank wird langsam mit großen Queries? | Indizes! `idx_cards_name` für LIKE-Suche, FTS5 für Volltextsuche |

### 6.2 SQLite FTS5 für Volltextsuche (Optionale Erweiterung)

```sql
-- Später: Für schnellere Kartensuche
CREATE VIRTUAL TABLE IF NOT EXISTS cards_fts USING fts5(
    name, 
    type_line,
    oracle_text,
    content=cards  -- "Contentless" FTS — referenziert Original-Tabellen
);

-- Query wird dann: SELECT * FROM cards WHERE name MATCH 'wizard' 
-- statt: SELECT * FROM cards WHERE name LIKE '%wizard%' (sehr langsam!)
```

---

## 7. Cargo.toml Dependencies (Final)

```toml
[package]
name = "mtg-multiverse-studio"
version = "0.1.0"  
edition = "2024"   # ← auf Edition 2024 updaten!

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
# === Tauri (Core) ===
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tauri = { version = "2", features = [
    "protocol-asset",      # Lokale Dateizugriffe für Lore-Markdown
    "path-all",            # Pfad-Umhüllungen (resolve path)
] }

# === Datenbank ===
rusqlite = { version = "0.31", features = ["bundled", "fts5"] }  ← fts5 Feature!

# === HTTP Client für Scryfall API ===  
reqwest = { version = "0.11", features = [
    "json",                # reqwest::get().json::<T>()
], default-features = false }

tokio = { version = "1", features = ["full"] }  ← Async Runtime!

# === Caching ===
lru = "0.12"   # LRU Cache für häufig abgefragte Karten

# === Markdown Parsing (für Lore) ===
pulldown-cmark = "0.9"     # Markdown → HTML Konvertierung  
frontmatter_parser = "1"   # YAML Frontmatter aus .md Dateien

# === Logging/Debugging ===
tracing = { version = "0.1", features = ["release_max_level_info"] }

[features]
custom-protocol = ["tauri/custom-protocol"]
default = ["custom-protocol"]
```

---

## 8. Frontend-Kommunikation (TypeScript-Seite)

### 8.1 Frontend ruft Commands auf

```typescript
// src/services/api.ts — Tauri invoke Wrapper

interface SearchArgs { query: string }

export async function searchCards(args: SearchArgs): Promise<SearchResult> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('search_cards', { args });  // ← exakt der Command-Name in commands.rs!
}

export async function getCard(id: string): Promise<CardResponse> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('get_card', { id });
}
```

### 8.2 Datenfluss-Beispiel (Kompletter Request-Zyklus)

```
1. User tippt "black lotus" in die Suche ein
   ↓
2. React: searchCards({ query: "black lotus" }) 
   → invoke('search_cards', { args: { query: "black lotus" } })
   ↓
3. Rust Backend (commands.rs):
   a) Zuerst SQLite fragen: SELECT * FROM cards WHERE name LIKE '%black lotus%'  
      → FOUND! → return immediately ✅
   
   b) Nicht in DB? → Scryfall API fragen (async, rate-limited!)
      → Ergebnis in SQLite speichern  
      → LRU Cache updaten
      → return
   ↓
4. Frontend erhält SearchResult { cards: [...], total: 3 }
5. React rendert CardPreview-Komponenten mit Bild-URLs von Scryfall
```

---

## 9. Nächste Schritte — Prioritäten

1. **Cargo.toml vervollständigen** (Dependencies + Features)  
2. **main.rs schreiben** (Tauri Setup, AppState, Command-Registrierung)
3. **Datenbank-Schema implementieren** (`db/schema.rs` + `001_initial.sql`)  
4. **Connection-Pool einrichten** (`db/connection.rs`)
5. **Scryfall Models definieren** (`scryfall/models.rs`) — mit Serde
6. **Scryfall HTTP-Client schreiben** (`scryfall/client.rs`)
7. **Repository-Pattern implementieren** (`db/card_repo.rs` etc.)  
8. **Services schichten** (`services/card_service.rs`)
9. **Commands definieren & registrieren** (`commands.rs`)
