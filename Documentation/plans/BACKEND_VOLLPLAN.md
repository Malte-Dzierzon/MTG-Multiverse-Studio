# Backend-Vollplan — MTG Multiverse Studio

> **Ziel:** Das Rust-Backend vollständig frontend-ready machen: Blitzschnelle Suche, Collection-/Deck-Management, Import von externen Sammlungen.

---

## Übersicht — 5 Arbeitspakete

```
┌─────────────────────────────────────────────────────┐
│                   FRONTEND (React)                   │
│  invoke('search_cards')  •  get_collection()  • …   │
└──────────────┬───────────────────────┬───────────────┘
               │ Tauri IPC (async)     │
┌──────────────▼───────────────────────▼───────────────┐
│              RUST BACKEND (Tauri Commands)            │
│  cards  │  collection  │  decks  │  import  │  sets  │
└──────────────┬───────────────────────┬───────────────┘
               │ rusqlite              │
┌──────────────▼───────────────────────▼───────────────┐
│              SQLITE (WAL + FTS5)                     │
│  cards  │  sets  │  collection  │  decks  │  …      │
└─────────────────────────────────────────────────────┘
```

---

## Phase 1: FTS5 + Blitzschnelle Suche

**Status:** `LIKE '%query%'` → Full-Table-Scan, 2-5s auf 38k Karten
**Ziel:** FTS5 → instant (< 50 ms)

### Änderungen
1. **Migration 002_fts5.sql** – FTS5-Tabelle + Trigger
2. **`db/schema.rs`** – `CREATE_VIRTUAL_TABLE` Konstante
3. **`bin/import.rs`** – FTS5 nach dem Bulk-Import aktualisieren
4. **`db/card_repo.rs`** – `search_cards_fts()` mit MATCH statt LIKE
5. **`services/card_service.rs`** – FTS5 zuerst, Fallback auf LIKE

### FTS5-Schema
```sql
CREATE VIRTUAL TABLE cards_fts USING fts5(
    name, type_line, oracle_text, flavor_text,
    content='cards', content_rowid='rowid',
    tokenize='unicode61'
);
```

### Query-Pattern
```rust
SELECT c.id, c.name, c.type_line, c.mana_cost, …
FROM cards_fts f JOIN cards c ON c.rowid = f.rowid
WHERE cards_fts MATCH :query
ORDER BY rank
LIMIT 50
```

### Ergebnisse
| Query | Vorher (LIKE) | Nachher (FTS5) |
|-------|--------------|----------------|
| `Opt` | ~500 ms | < 5 ms |
| `draw a card` | ~3 s | < 20 ms |
| `commander AND flying` | — | < 10 ms |

---

## Phase 2: Sets anreichern + Datenqualität

**Problem:** `sets`-Tabelle leer, `set_name` in Cards immer leer

### Lösung
1. **Import-Sets aus Scryfall** (`/bulk-data` liefert keine Sets, aber `/sets`)
2. **`import_engine` erweitern** – `cargo run --bin mtg-import -- sets` lädt alle Sets
3. **Card-Response patchen** – `set_name` per JOIN aus `sets`-Tabelle (statt leerem String)

### Sets-API
```rust
#[command]
pub fn list_sets(app: …) -> Result<Vec<SetResponse>>;
#[command]
pub fn get_set(app: …, id: String) -> Result<SetResponse>;
```

---

## Phase 3: Collection-Management

**Aktuell:** Nur `add_to_collection` existiert, keine Abfrage, kein Löschen.

### Neue Commands
| Command | Beschreibung |
|---------|-------------|
| `get_collection` | Alle Karten in Sammlung + Paginierung |
| `update_collection_item` | Menge, Condition ändern |
| `remove_from_collection` | Karte entfernen |
| `search_collection` | Nur in Sammlung suchen (FTS5 + JOIN) |

### Collection-Import (Dateien)
| Format | Beschreibung |
|--------|-------------|
| **CSV (generic)** | `card_name, quantity, condition, set_code` |
| **MTG Arena** | `.txt`-Export mit `1 Card Name (SET) XXX` |
| **Moxfield** | `.csv/.json` mit Deck-Export |
| **Archidekt** | `.json` mit Deck-Export |

### Import-API
```rust
#[command]
pub fn import_collection_csv(app: …, data: String) -> Result<ImportResult>;
// Erwartet: name|quantity|set_code durch Komma/Tab getrennt
```

---

## Phase 4: Deck-Management verbessern

**Aktuell:** Nur `create_deck`, `get_deck`, `list_decks`, `get_deck_mana_curve`

### Fehlende Commands
| Command | Beschreibung |
|---------|-------------|
| `add_card_to_deck` | Karte + quantity zu Deck |
| `remove_card_from_deck` | Karte aus Deck entfernen |
| `update_deck` | Name/Format/Beschreibung ändern |
| `delete_deck` | Deck löschen + Karten-Referenzen |
| `search_decks` | Decks nach Name durchsuchen |
| `get_deck_colors` | Farb-Identität eines Decks |
| `validate_deck` | Format-Legalität prüfen |

### DeckCard erweitern
```sql
ALTER TABLE deck_cards ADD COLUMN category TEXT DEFAULT 'mainboard';
-- 'mainboard', 'sideboard', 'maybe'
```

---

## Phase 5: Performance & Architektur

### Sofort umsetzbar
1. **`synchronous=NORMAL`** ✅ bereits erledigt
2. **WAL-Mode** bereits in 001_initial.sql gesetzt ✅
3. **Prepared-Statements cachen** – `conn.prepare()` einmalig, wiederverwenden
4. **N+1 in `list_decks`** → Batch-Query statt Einzel-Abfrage pro Deck
5. **Struct-Duplikate auflösen** – `commands.rs`-Args durch `models.rs`-Importe ersetzen

### Schema-Konsolidierung
Die `cards`-Tabelle hat jetzt 32 Spalten via ALTER TABLE. Nächste Migration sollte die Spalten in das ursprüngliche CREATE TABLE überführen:
```sql
CREATE TABLE cards_v2 (...alle 32 spalten...);
INSERT INTO cards_v2 SELECT * FROM cards;
DROP TABLE cards;
ALTER TABLE cards_v2 RENAME TO cards;
```

---

## Frontend-Bindung (Tauri IPC)

Jeder Command ist async (oder blocking via `tauri::async_runtime::spawn_blocking`):
```rust
#[command]
pub async fn search_cards(args: SearchCardsArgs) -> Result<SearchResult> {
    let db = STATE.db.lock()?;
    // …
}
```

Wichtige Pattern:
- **Loading States** – Frontend zeigt Spinner, Backend antwortet in < 50 ms
- **Pagination** – `offset` + `limit` in jedem Search-Command
- **Thumbnails** – `image_url_small` direkt aus DB, kein zusätzlicher Fetch
- **Debounced Search** – Frontend debounced 300ms, Backend FTS5

---

## Datenmodell — Aktuell vs. Ziel

```diff
 cards:
   + flavor_text TEXT
   + layout TEXT
   + set_code TEXT
-  set_id TEXT REFERENCES sets(id)
   + set_name TEXT
   + set_type TEXT
   + collector_number TEXT
   + power TEXT / toughness TEXT / loyalty TEXT / defense TEXT
   + edhrec_rank REAL
   + finishes TEXT
   + frame TEXT / border_color TEXT
   + released_at TEXT
-  image_uris TEXT
   + image_uris_json TEXT

 sets:
   id TEXT PRIMARY KEY           ✅ vorhanden
   name TEXT NOT NULL            ✅
   set_type TEXT                 ✅
   released_at DATE              ✅
   icon_svg_uri TEXT             ➕ neu
   scryfall_uri TEXT             ➕ neu
   card_count INTEGER            ➕ neu

 deck_cards:
   + category TEXT DEFAULT 'mainboard'  (mainboard/sideboard/maybe)

 collection:
   + language TEXT DEFAULT 'en'
   + is_foil INTEGER DEFAULT 0
   + acquired_at DATE
```

---

## Sub-Agent Aufgaben

### Sub-Agent 1: FTS5 + Search
- Migration 002_fts5.sql
- FTS5-Trigger in schema.rs
- `search_cards_fts()` in card_repo.rs
- FTS5-Nachbefüllung in import.rs
- `list_sets` + Sets aus Scryfall importieren

### Sub-Agent 2: Collection + Import
- `get_collection`, `update_collection`, `remove_from_collection`
- `search_collection` (FTS5 + JOIN)
- CSV/MTGA/Moxfield/Archidekt Parser
- `import_collection_*` Commands
- Collection-Schema erweitern (language, is_foil, acquired_at)

### Sub-Agent 3: Deck-Erweiterung + Optimierung
- `add_card_to_deck`, `remove_card_from_deck`
- `update_deck`, `delete_deck`
- `validate_deck` (Format-Legalität)
- `category`-Spalte in deck_cards
- N+1-Fix in `list_decks`
- Struct-Duplikate bereinigen (commands.rs → models.rs)

---

## Meilensteine

1. ✅ FTS5 aktiviert → Suche in < 10 ms
2. ✅ Sets-Tabelle befüllt → `set_name` im Frontend sichtbar
3. ✅ Collection CRUD → Vollständige Sammlungsverwaltung
4. ✅ Collection Import → Batch-Import aus CSV/MTGA/Moxfield
5. ✅ Deck CRUD → Karten hinzufügen/entfernen, Sideboard, Validation
6. ⬜ Architecture cleanup → N+1, Duplikate, Preped-Statements
