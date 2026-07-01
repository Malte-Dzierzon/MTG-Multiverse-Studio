# Architecture & Data Flow Review

> **Projekt:** MTG Multiverse Studio (Tauri v2 + Rust + SQLite)  
> **Datum:** 2026-07-01  
> **Review-Tiefe:** Vollständiger Code-Durchlauf aller Rust-Module, SQL-Schema, API-Integration, Tauri-IPC  
> **Geprüfte Pfade:** `src-tauri/src/` (alle 20 Dateien), `Documentation/IMPORT_STRATEGY.md`

---

## 1. DB-Schema

### 1.1 Tabellenstruktur (aktuell)

| Tabelle | Primärschlüssel | Fremdschlüssel | Indexe |
|---------|----------------|----------------|--------|
| `sets` | `id TEXT PK` | – | – |
| `cards` | `id TEXT PK` | `set_id → sets(id)` | 4 (name, oracle_id, set_rarity, cmc) |
| `collection` | `id INTEGER PK AUTO` | `card_id → cards(id) UNIQUE` | 1 (card_id) |
| `decks` | `id INTEGER PK AUTO` | – | – |
| `deck_cards` | `id INTEGER PK AUTO` | `deck_id → decks(id) CASCADE`, `card_id → cards(id)` | 3 (deck, card, lookup) |
| `lore_entries` | `id INTEGER PK AUTO` | – | 2 (type, title) |

### 1.2 Gefundene Probleme

#### 🚨 FTS5 ist deaktiviert
- `CREATE_CARDS_FTS` und die Sync-Trigger sind in `schema.rs` **auskommentiert** (Zeile 155–157).
- Die Migrations-SQL `001_initial.sql` hat FTS5 ebenfalls auskommentiert.
- **Konsequenz:** `search_cards_by_name()` verwendet `LIKE '%...%'` — ein Full-Table-Scan, der ab ~50.000 Karten spürbar langsamer wird. Bei 100.000+ Prints inakzeptabel.
- **Fix:** FTS5 aktivieren. `rusqlite` in Cargo.toml benötigt das `fts5` Feature. Die Trigger (`cards_ai`, `cards_ad`, `cards_au`) sind korrekt formuliert.

#### 🚨 `LIKE '%...%'` kann Index nicht nutzen
- `idx_cards_name` auf `cards(name)` hilft **nicht** bei `WHERE name LIKE '%query%'` (führender Wildcard).
- Nur `LIKE 'query%'` (Präfix-Suche) würde den B-Tree-Index nutzen.
- **Alternative:** FTS5 für Volltext-Suche aktivieren (s.o.).

#### 🚨 `set_name` wird nie befüllt
- In `card_db_to_response()` (card_repo.rs, Zeile 161):  
  `set_name: String::new()` → **immer leer**.
- Die `sets`-Tabelle hat den Namen, aber es gibt `kein JOIN` in `get_card_by_id()` oder `search_cards_by_name()`.
- **Fix:** Query um `LEFT JOIN sets ON cards.set_id = sets.id` ergänzen, `sets.name` als `set_name` zurückgeben.

#### ⚠️ JSON in TEXT-Spalten — Parser-Overhead auf jedem Read
- `colors`, `color_identity`, `keywords`, `image_uris`, `legalities`, `prices` werden als JSON-Strings gespeichert.
- `card_db_to_response()` ruft pro Karte 6× `serde_json::from_str()` auf → ~6 µs pro Karte laut Benchmarks. Bei 50 Suchergebnissen ~300 µs, bei Bulk-Export addiert sich das.
- **Alternative:** `rusqlite` bietet kein nativen JSON-Typ. Bleibt so, aber `image_uris.small` und `image_uris.large` könnten als separate Spalten `image_small_url TEXT, image_large_url TEXT` ausgelagert werden (häufig abgefragt, selten geändert).

#### ⚠️ Collection: `UNIQUE(card_id)` verhindert Mehrfach-Einträge
- Ein User kann eine Karte nur **einmal** in der Sammlung haben (quantitativ, nicht konditionsgetrennt).
- Möchte man z.B. eine Karte in "NM" und eine in "HP" getrennt führen, ist das nicht möglich.
- **Bewertung:** Für ein Sammel-Tool ist das ein Design-Feature, kein Bug. Falls später conditions-getrennte Inventorys gewünscht sind, muss das UNIQUE auf `(card_id, condition)` erweitert werden.

#### ⚠️ Fehlende Timestamps
- `cards` hat kein `last_synced_at` oder `updated_at` → kein inkrementelles Update möglich.
- `collection` hat nur `added_at`, kein `updated_at`.
- **Betroffen:** Inkrementelle Syncs mit Scryfall (`updated_at`-Feld in Bulk-Data).

#### ✅ Korrekt
- `deck_cards` mit `ON DELETE CASCADE` auf `decks(id)`.
- `idx_deck_cards_lookup` Composite-Index `(deck_id, card_id)` für Upsert-Performance.
- WAL-Mode + `foreign_keys = ON` in `init_schema()` und `init_db()`.
- `busy_timeout = 5000` verhindert SQLITE_BUSY-Fehler.

---

## 2. Caching-Strategie

### 2.1 Behauptet: 3-Stufen-Cache (DB → LRU → Scryfall)

Die Dokumentation (`card_service.rs` Header, `IMPORT_STRATEGY.md`) beschreibt:

```
Tier 1: SQLite (offline-fähig, persistent)
Tier 2: LRU Cache (in-memory, schnell)
Tier 3: Scryfall API (langsam, rate-limited)
```

### 2.2 Tatsächliche Implementierung

```
           ┌──────────────────────────────────────────────────┐
           │                AppState                          │
           │  ┌─────────────────────┐  ┌──────────────────┐   │
           │  │ db: Mutex<Connection>│  │ scryfall_client: │   │
           │  │                     │  │ Mutex<SFClient>  │   │
           │  └────────┬────────────┘  └────────┬─────────┘   │
           │           │                        │             │
           └───────────┼────────────────────────┼─────────────┘
                       │                        │
              ┌────────▼────────┐      ┌────────▼─────────┐
              │  card_repo.rs   │      │ ScryfallClient    │
              │  (SQLite only)  │      │ ┌──────────────┐ │
              │                 │      │ │ LruCache(1000)│ │
              │                 │      │ │ (nur für      │ │
              │                 │      │ │ get_card_by_id│ │
              │                 │      │ │   & scoped)   │ │
              └─────────────────┘      │ └──────────────┘ │
                                       │ RateLimiter:     │
                                       │ Semaphore(10)    │
                                       └──────────────────┘
```

### 2.3 Kritische Probleme

#### 🚨 **Kein Service-Level-Cache**
- Die `card_service`-Schicht hat **keinen eigenen LRU-Cache**. Der LRU-Cache lebt nur im `ScryfallClient`.
- Der Pfad: `card_service::get_card()` → prüft DB → **überspringt LRU völlig** → ruft API.
- Ein Treffer im LRU-Cache spart nur den API-Call, nicht den DB-Call.

#### 🚨 **ScryfallClient-Mutex hält Lock über async hinweg (gefährlich)**
- `ScryfallClient` ist in `AppState` als `Mutex<ScryfallClient>`.  
- Alle Commands holen `state.scryfall_client.lock().unwrap()`. Da die Commands aber **synchron** sind (siehe §4), passiert das aktuell nicht.
- Wenn async-Commands implementiert werden: `lock()` hält den Mutex über `.await`-Points → **Deadlock-Gefahr** oder massive Kontention.
- **Fix:** `tokio::sync::Mutex` für den ScryfallClient, oder `Arc<RwLock<ScryfallClient>>`.

#### 🚨 **LRU-Cache inkonsistent genutzt**
- `get_card_by_id()`: ✅ Prüft Cache → API → speichert in Cache
- `get_card_by_name()`: ❌ **Prüft Cache nicht**, geht immer zur API → speichert Ergebnis nur im Cache
- `search_cards()`: ❌ **Prüft Cache nicht**, geht immer zur API → speichert nichts im Cache
- **Fix:** Einheitliches Cache-Check-Muster für alle Methoden.

#### ⚠️ **Kein invalidation-Mechanismus**
- Der LRU-Cache (1000 Einträge) hat kein TTL/Expiry. Ein einmal gecachtes `ScryfallCard` bleibt bis zur Verdrängung durch LRU erhalten.
- Scryfall-Daten ändern sich täglich (Preise, Errata). Ohne TTL können Preise im Cache veralten.
- **Fix:** `lru::LruCache` mit optionalem TTL-Wrapper oder `cached`-Crate in Betracht ziehen.

#### ⚠️ **KB-Größe pro Cache-Eintrag**
- `ScryfallCard` hat ~50 Felder inkl. `oracle_text`, `image_uris`, `legalities`, `prices`. Ein serialisiertes `ScryfallCard` ist ~2-5 KB.
- 1000 Einträge × ~3 KB = ~3 MB RAM. Das ist **akzeptabel**, aber bei 10.000 Einträgen wären es 30 MB — für eine Desktop-App immer noch im Rahmen.

### 2.4 Empfohlene Architektur

```
┌──────────┐    ┌──────────────────┐    ┌───────────────┐    ┌──────────────┐
│ Frontend │───▶│  Tauri Command   │───▶│  CardService   │───▶│  SQLite      │
│ (React)  │    │  (synchron oder  │    │  (Business     │    │  (Tier 1)    │
│          │    │   async)         │    │   Logic)       │    │              │
└──────────┘    └──────────────────┘    │               │    └──────────────┘
                                         │ ┌───────────┐│         │
                                         │ │ LruCache   ││◀────────┘ (Fallback)
                                         │ │ (5000,TTL) ││───▶ ScryfallClient
                                         │ └───────────┘│    ┌──────────────┐
                                         │               │    │ API (Tier 3) │
                                         └───────────────┘    └──────────────┘
```

**Prinzip:** Der Service hält einen `LruCache<CardResponse>` (nicht `ScryfallCard`).  
**Flow:** Frontend → Service → prüft LRU → prüft DB → prüft API → speichert in DB + LRU.

---

## 3. API-Integration (Scryfall)

### 3.1 Rate-Limiting: ❌ Falsch implementiert

| Aspekt | Soll | Ist |
|--------|------|-----|
| Limit | 10 req/s | `Semaphore::new(10)` |
| Burst | 100 req/min (soft limit) | Nicht implementiert |
| 429-Handling | Retry with backoff | Nicht implementiert |
| 5xx-Handling | Retry 3x with backoff | Nicht implementiert |

`Semaphore::new(10)` erlaubt **10 gleichzeitige Requests**, aber **keine Rate pro Sekunde**.  
Ohne Zeitsteuerung können theoretisch 10 Requests in 1 ms gefeuert werden, der 11. wartet unendlich (Semaphore blockiert).

**Fix:** Token-Bucket-Algorithmus (z.B. `governor`-Crate) oder `tokio::time::sleep` nach jedem Request.

### 3.2 Fehlende Features

#### 🚨 Kein Streaming-Support
- `response.json().await?` deserialisiert den gesamten Body in den Speicher.
- Für Bulk-Import (~180 MB Oracle Cards) würde das ~180 MB RAM kosten + Heap-Fragmentierung.
- **Fix:** `reqwest` mit `stream`-Feature aktivieren, `bytes_stream()` verwenden und per `serde_json::Deserializer::from_reader(stream).into_iter::<ScryfallCard>()` streamen.

#### 🚨 Keine Paginierung in `search_cards()`
- Die Methode gibt nur **eine Seite** zurück (max. 175 Karten).
- Scryfall signalisiert via `has_more: true` + `next_page: URL`, dass weitere Seiten existieren.
- Der Service kümmert sich nicht darum → unvollständige Suchergebnisse.
- **Fix:** `search_cards()` sollte `has_more` prüfen und ggf. `next_page` folgen (mit konfigurierbarem Limit).

#### ⚠️ Kein Error-Detail-Parsing
- Scryfall gibt detaillierte Fehler-JSONs zurück: `{"status": 404, "code": "not_found", "details": "Card not found"}`.
- Der Code wirft nur `AppError::Unknown(format!("Scryfall API error {} ..."))`, ohne den Body zu lesen.
- **Fix:** `response.error_for_status_ref()?` → bei Fehler: `response.text()` parsen und strukturierten Fehler zurückgeben.

#### ⚠️ `reqwest` ohne TLS-Backend
- `default-features = false` → kein `rustls` oder `native-tls`.
- Auf Systemen ohne installiertes OpenSSL kann der HTTPS-Request fehlschlagen.
- **Fix:** `features = ["json", "rustls-tls"]` oder `features = ["json", "native-tls"]`.

### 3.3 Stärken
- ✅ User-Agent gesetzt (`mtg-multiverse-studio/0.1.0`) — Scryfall-Richtlinie erfüllt.
- ✅ `ScryfallClient` ist `Clone`-bar via `Arc`-internals.
- ✅ Async-API (tokio) — blockiert nicht den Tauri-Hauptthread (wenn async verwendet wird).

---

## 4. Tauri-IPC Commands

### 4.1 Command-Übersicht

| Command | Args | Returns | Sync? | Async-Fallback? |
|---------|------|---------|-------|-----------------|
| `search_cards` | `SearchCardsArgs` | `SearchResult` | ✅ Sync | ❌ Only local DB |
| `get_card` | `GetCardArgs` | `CardResponse` | ✅ Sync | ❌ Only local DB |
| `add_to_collection` | `AddToCollectionArgs` | `CollectionItemResponse` | ✅ Sync | – |
| `create_deck` | `CreateDeckArgs` | `CreatedDeckResponse` | ✅ Sync | – |
| `get_deck` | `GetCardArgs` (sic!) | `DeckResponse` | ✅ Sync | – |
| `list_decks` | – (none) | `Vec<DeckResponse>` | ✅ Sync | – |
| `load_lore_entries` | `LoadLoreArgs` | `Vec<LoreEntryResponse>` | ✅ Sync | – |
| `get_deck_mana_curve` | `GetCardArgs` (sic!) | `serde_json::Value` | ✅ Sync | – |

### 4.2 Kritische Probleme

#### 🚨 **Alle Commands sind synchron — Scryfall-Client tot**
- Kein einziges Command ruft `ScryfallClient`-Methoden auf (alle sind async).
- Die `card_service::get_card()` (async) und `card_service::search_cards()` (async) werden von keinem Command verwendet.
- Die `scryfall_client` in `AppState` wird beim Start initialisiert und **nie wieder angefasst**.
- **Konsequenz:** Der gesamte 3-Stufen-Cache (DB → LRU → Scryfall) ist unvollständig. Es gibt nur Tier 1 (DB).

#### 🚨 **Typ-Sicherheit: `GetCardArgs` wird für Deck-ID missbraucht**
- `get_deck` und `get_deck_mana_curve` benutzen `GetCardArgs { id: String }`.
- Die Deck-ID muss zur Laufzeit von `String` nach `i64` geparst werden.
- Ein Tippfehler im Frontend (z.B. `"abc"` statt Deck-ID) crasht das Command mit `AppError::Validation`.
- **Fix:** Eigene Args-Structs: `GetDeckArgs { deck_id: i64 }` — Tauri deserialisiert direkt.

#### 🚨 **`list_decks`: N+1-Queries**
- `list_decks` ruft für jedes Deck `get_deck_with_cards()` auf, das wiederum `get_card_by_id()` für jede Karte im Deck aufruft.
- Bei 10 Decks mit je 60 Karten = **1 + 10 + 600 = 611 SQL-Queries**.
- **Fix:** Batch-Query mit `JOIN` und `GROUP_CONCAT`, oder separate `get_deck_summary()`-Funktion die nur Metadaten lädt, Karten lazy.

#### 🚨 **`add_to_collection` gibt hartkodierte Werte zurück**
- `id: 0` — Das `collection_repo::add_to_collection()` liefert die echte `rowid`, aber das Command ignoriert sie.
- `notes: None` — Wird nicht aus der DB gelesen (die DB hat keinen Notes-Parameter für Insert, aber `update_collection_item` hat einen).
- `added_at: chrono::Utc::now()` — Nicht der tatsächliche DB-Timestamp.
- **Fix:** Nach dem Insert: `get_collection_item(id)` aufrufen und das echte DB-Objekt zurückgeben.

### 4.3 Fehlende Commands

#### Standard-CRUD für Collection
- ❌ `get_collection` — Auflistung aller Sammlungs-Items
- ❌ `update_collection_item` — Menge/Zustand/Notizen ändern
- ❌ `remove_from_collection` — Karte aus Sammlung entfernen

#### Standard-CRUD für Decks
- ❌ `update_deck` — Deck-Metadaten ändern
- ❌ `delete_deck` — Deck löschen
- ❌ `add_card_to_deck` — Karte ins Deck einfügen
- ❌ `remove_card_from_deck` — Karte aus Deck entfernen

#### Scryfall-Integration
- ❌ `import_oracle_cards` — Bulk-Import starten
- ❌ `get_import_progress` — Fortschritt des Imports abfragen
- ❌ `sync_prices` — Preise aktualisieren

#### Suche & Filter
- ❌ `search_cards_with_scryfall` — Auch API durchsuchen
- ❌ `advanced_search` — Nach CMC, Farbe, Set, Rarity filtern

### 4.4 Response-Typen optimieren

- `SearchResult::from_cache` ist **immer `true`**, weil es nie eine API-Suche gibt — das Feld ist irreführend.
- `DeckResponse` lädt **immer alle Karten**, auch wenn nur Metadaten gebraucht werden → Separates `DeckSummary`-Modell einführen.
- `get_deck_mana_curve` gibt `serde_json::Value` zurück statt einem typisierten Struct `ManaCurveResponse`.

---

## 5. Datenfluss (End-to-End)

### 5.1 Such-Flow (aktuell)

```
Frontend invoke('search_cards', {query: "Opt"})
  │
  ▼
commands::search_cards()          [sync, Mutex::lock]
  │
  ▼
card_service::search_cards_local()
  │
  ▼
card_repo::search_cards_by_name()
  ├── SQL: SELECT ... WHERE name LIKE '%Opt%' LIMIT 50
  └── 6× JSON.parse pro Karte (colors, image_uris, legalities, prices, ...)
  │
  ▼
API-Fallback?                     ❌ FEHLT
LRU-Cache?                        ❌ FEHLT
  │
  ▼
Return SearchResult               [from_cache: true — irreführend]
```

### 5.2 Karten-Detail-Flow (aktuell)

```
Frontend invoke('get_card', {id: "uuid"})
  │
  ▼
commands::get_card()              [sync, Mutex::lock]
  │
  ▼
card_repo::get_card_by_id()
  ├── SQL: SELECT ... WHERE id = :id
  ├── Kein JOIN auf sets → set_name immer leer
  └── 6× JSON.parse
  │
  ▼
Nicht in DB?                      ❌ Sofort 404 — kein API-Fallback
```

### 5.3 Bewertung

Der aktuelle Datenfluss ist **funktional aber ineffizient**:

1. **Zu viele JSON-Parses** auf dem Response-Pfad. `card_db_to_response()` parst 6 JSON-Felder, baut dann JSON-Objekte wieder zusammen. Besser: `CardDb` speichert `colors` als `Vec<String>`, `image_uris_small` als `Option<String>` etc. — dann kein JSON-Parse auf dem Lesepfad.
2. **Kein API-Fallback** — die versprochene 3-Stufen-Architektur ist nur auf dem Papier.
3. **N+1 im Deck-Flow** — `list_decks` skaliert nicht.
4. **Mutex-Kontention** — alle Commands serialisieren auf `state.db` trotz WAL-Mode (der Concurrent Reads erlaubt).

### 5.4 Optimierter Flow (Vorschlag)

```
Frontend
  │
  ▼
commands::search_cards(query, page, use_api)
  │  Tauri::command + async
  ▼
card_service::search_cards(query, limit)
  │
  ├─ 1) Try LruCache<QueryResult>  (TTL: 5 min)
  │   → Treffer? Sofort zurück
  │
  ├─ 2) Try SQLite FTS5
  │   → Treffer? Cache + zurück
  │
  └─ 3) Scryfall API
      → rate-limited, mit retry
      → Ergebnisse in DB speichern
      → Cache + zurück
```

---

## 6. IMPORT_STRATEGY.md — Machbarkeitsprüfung

### 6.1 Was die Strategie beschreibt

| Stufe | Beschreibung | Quelle |
|-------|-------------|--------|
| 1 | Kartendaten (JSON) → SQLite | `data.scryfall.io/oracle-cards/...json` |
| 2 | Bilder on-demand streamen | `cards.scryfall.io` CDN |
| 3 | Preise von Cardmarket cachen | Cardmarket API (key erforderlich) |

### 6.2 Machbare Elemente

| Aspekt | Bewertung |
|--------|-----------|
| **Batch-Inserts mit Transaktion** | ✅ Gut. `100er-Batches in einer Transaktion` ist korrekt. |
| **PRAGMA-Optimierungen** | ✅ `cache_size = -80000` ist gut. `synchronous = OFF` ist riskant → besser `NORMAL`. |
| **Hybrid-Bildstrategie** | ✅ Standard: Scryfall CDN, optional lokaler Cache. Solide. |
| **Fortschritt via tracing** | ✅ Korrekt. |

### 6.3 Nicht realisierbare / falsche Annahmen

#### 🚨 **JSON ≠ JSONL**
- Die Strategie sagt "JSONL.gz" und "zeilenweises Parsen", aber Scryfalls Bulk-Data für Oracle Cards ist ein **JSON-Array** (`[{...}, {...}, ...]`), nicht Newline-Delimited JSON.
- **Konsequenz:** Zeilenweises Parsen funktioniert nicht. Man muss `serde_json::Deserializer::from_reader(reader).into_iter::<ScryfallCard>()` verwenden (Stream-Deserializer).

#### 🚨 **`serde_json::from_reader(reader)` lädt ALLES in RAM**
- Der Pseudocode in §2 zeigt:
  ```rust
  let deser = Deserializer::from_reader(reader);
  let cards: Vec<ScryfallCard> = ScryfallCard::deserialize(deser)?;
  ```
- **Das ist NICHT streaming!** `Vec<ScryfallCard>` deserialisiert das gesamte Array. Bei 180 MB JSON entspricht das ~180 MB + Heap-Buchhaltung.
- **Fix:** `Deserializer::from_reader(reader).into_iter::<ScryfallCard>()` → liefert `Iterator<Item=Result<ScryfallCard>>`.

#### 🚨 **`reqwest` ohne `stream`-Feature**
- `Cargo.toml` hat `reqwest = { features = ["json"], default-features = false }`.
- `response.bytes_stream()` (für Streaming) benötigt das Feature `stream`.
- Ohne `stream` kann man keine Streaming-Downloads machen → 180 MB müssen komplett im RAM landen.

#### 🚨 **`Connection` ist nicht `Send` — async-Konflikt**
- `rusqlite::Connection` implementiert `!Send` (es sei denn, es wird mit `#[cfg(feature = "send")]` kompiliert).
- `async fn import_oracle_cards(db: &rusqlite::Connection, ...)` würde nicht kompilieren, weil der `Future`-Generator `Send` erfordert.
- **Fix:** Die Funktion muss `db: &Mutex<Connection>` akzeptieren oder `&Connection` mit der `send`-Feature von rusqlite.

#### 🚨 **`frontmatter`-Crate ungenutzt**
- `serde_yaml` und `frontmatter` sind in Cargo.toml, aber Lore-Service hat **eigene YAML-Parser-Implementierung** (ca. 30 Zeilen, fehleranfällig).
- Die Strategie erwähnt das nicht. `frontmatter` ist tote Dependency (~5 KB Binary-Overhead).

#### 🚨 **Kein `updated_at`-Tracking für Bulk-Import**
- Scryfall Bulk-Data hat `updated_at` und `digest` im Metadaten-Objekt. Die Strategie erwähnt keinen Mechanismus, um zu prüfen, ob sich die Daten seit dem letzten Import geändert haben.
- Jedes App-Start würde ~180 MB downloaden, selbst wenn nichts geändert ist.

### 6.4 Änderungen im Code-Code

| Datei | Änderung |
|-------|----------|
| `Cargo.toml` | `reqwest` um `stream` und `rustls-tls` ergänzen. `rusqlite` um `fts5` ergänzen. `frontmatter` entfernen oder verwenden. |
| `db/migrations/001_initial.sql` | FTS5-Virtual-Table + Trigger **einkommentieren**. |
| `db/schema.rs` | `CREATE_CARDS_FTS` + Trigger aktivieren. |
| `db/connection.rs` | Migration auf `001_initial.sql` → `002_fts5.sql` aufteilen (abwärtskompatibel). |
| `db/card_repo.rs` | `search_cards_by_name()` durch `search_cards_fts()` ergänzen. `card_db_to_response()` um `JOIN sets.name AS set_name` ergänzen. |
| `scryfall/client.rs` | Rate-Limiter auf Token-Bucket umstellen. Streaming-Methoden für Bulk-Import ergänzen. |
| `services/card_service.rs` | Service-Level LruCache einbauen. `get_card()` und `search_cards()` auch async von Commands nutzbar machen. |
| `commands.rs` | 8 bestehende Commounds async machen. Neue Commands ergänzen (Collection-CRUD, Deck-CRUD, Import). |
| `models.rs` | `GetDeckArgs { deck_id: i64 }`, `ManaCurveResponse`-Struct, `DeckSummary` ohne Karten. `CardDb` um `set_name` ergänzen. |
| `main.rs` | `scryfall_client` in `AppState` mit `tokio::sync::Mutex` austauschen. |

---

## 7. Verbesserungsvorschläge (zusammenfassend)

### P0 — Kritisch (muss vor Release)

1. **FTS5 aktivieren**: `rusqlite` mit `fts5`-Feature, `CREATE_VIRTUAL_TABLE`, Trigger einkommentieren.
2. **Commands async machen + API-Fallback**: `get_card` und `search_cards` müssen `ScryfallClient` nutzen.
3. **Rate-Limiter korrigieren**: Token-Bucket statt Semaphore.
4. **`set_name` via JOIN befüllen**: `LEFT JOIN sets ON cards.set_id = sets.id`.
5. **`reqwest` TLS-Feature**: `features = ["json", "stream", "rustls-tls"]`.

### P1 — Hoch (Performance / UX)

6. **`list_decks` N+1 eliminieren**: Batch-Join oder separaten Lightweight-Endpunkt.
7. **Service-Level LRU-Cache**: `card_service` hält `LruCache<CardResponse>` zwischen DB und API.
8. **PRAGMA synchronous = NORMAL** im Import, nicht OFF.
9. **`GetCardArgs` vs `GetDeckArgs`**: Typ-sichere Args-Structs.
10. **`add_to_collection` Response korrigieren**: Echte DB-Werte zurückgeben.

### P2 — Mittel (Architektur / Wartbarkeit)

11. **`frontmatter`-Crate entfernen** oder verwenden.
12. **Bulk-Import streaming** mit `serde_json::StreamDeserializer`.
13. **`_meta`-Tabelle** für Last-Sync-Timestamp + Digest.
14. **Missing CRUD-Commands** ergänzen (Collection-Update/Delete, Deck-Update/Delete/Cards).
15. **`serde_json::Value` in Response durch typisierte Structs ersetzen** (ManaCurve, ColorBalance).

### P3 — Nice-to-have

16. **`image_small_url`/`image_large_url` als separate Spalten** (spart JSON-Parse bei jedem CardResponse).
17. **Tokio::sync::Mutex für ScryfallClient** (verhindert Lock-über-async-Probleme).
18. **Scryfall-Error-Detail-Parsing** für bessere Fehlermeldungen.
19. **Preisverlauf-Tabelle** (`price_history`) wie in IMPORT_STRATEGY.md skizziert.
20. **ETag/Cache-Control von Scryfall nutzen**.

---

## 8. Fazit

Der Backend-Code hat eine **solide Grundstruktur** (modular, kommentiert, Tests vorhanden).  
Das **Architekturversprechen** (3-Stufen-Cache, Scryfall-Integration, Bulk-Import) ist aber in der aktuellen Implementierung **nur zu ~30% erfüllt**:

- ✅ DB-Schema: Funktionell, aber FTS5 deaktiviert → `LIKE`-Suche suboptimal.
- ❌ Caching: Service-Level-Cache fehlt → Nur Tier 1+3 ohne Tier 2.
- ❌ API-Integration: Rate-Limiter falsch, kein Streaming, kein Retry.
- ❌ Tauri-IPC: `ScryfallClient` nie genutzt → API-Fallback-Tot. N+1 in Deck-Queries.
- ⚠️ IMPORT_STRATEGY.md: Architektonisch sinnvoll, aber der Code-Pseudocode enthält einen **Stop-the-World-Fehler** (alles in RAM laden statt echtes Streaming).

**Empfehlung:** Vor dem nächsten Feature-Entwicklungsschub sollten die P0- und P1-Punkte abgearbeitet werden, insbesondere die Aktivierung von FTS5 und die async-Umstellung der Commands mit API-Fallback.
