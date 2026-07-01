# Code Optimization Review — Rust Backend

> **Projekt:** MTG Multiverse Studio (Tauri v2 + Rust)  
> **Review-Datum:** 2026-07-01  
> **Geprüfte Dateien:** 19 `.rs` Dateien (db/*, services/*, scryfall/*, commands.rs, main.rs, models.rs, utils/*)  
> **Status:** Phase 1 (Grundfunktionen) implementiert, diverse Optimierungspotenziale identifiziert

---

## Inhaltsverzeichnis

1. [Gefundene Issues (mit File:Line Referenzen)](#1-issues)
2. [Performance-Empfehlungen](#2-performance)
3. [Sicherheits-Check](#3-sicherheit)
4. [Nächste Optimierungsschritte](#4-optimierungsschritte)

---

## 1. Gefundene Issues

### 1.1 Performance

#### 🟠 P1 — N+1 Queries in Deck-Operationen

**Fundort:** `commands.rs:164-167` (`list_decks`)  
**Fundort:** `commands.rs:211-218` (`get_deck_mana_curve`)  
**Fundort:** `deck_service.rs:45-53` (`get_deck_with_cards`)

```rust
// commands.rs:164-167
for d in decks {
    let deck = deck_service::get_deck_with_cards(&db, d.id)?;  // N+1!
    result.push(deck);
}
```

**Problem:** Für jedes Deck wird ein separater Query ausgeführt (`get_deck_by_id` + `get_deck_cards` + pro Karte ein `get_card_by_id`). Bei 50 Decks mit je 60 Karten → **~3.050 statt 3 Queries**.

**`deck_service.rs:45-53`:** Gleiches Muster — `deck_cards` werden einzeln über `get_card_by_id` aufgelöst, statt mit einem `JOIN` in einem Query.

**`commands.rs:211-218` (`get_deck_mana_curve`):** Lädt erst alle `deck_cards`, dann per `card_repo::get_card_by_id` jede einzelne Karte — wieder N+1.

**Gegenmaßnahme:** JOIN-Query in `deck_repo` (siehe Performance-Empfehlungen).

---

#### 🟠 P2 — LIKE mit führendem Wildcard

**Fundort:** `db/card_repo.rs:85`

```rust
let like_pattern = format!("%{}%", name);
```

**Problem:** `LIKE '%query%'` kann den Index auf `cards(name)` nicht nutzen → Full-Table-Scan. FTS5 ist in `schema.rs:113-121` und der Migration (`001_initial.sql:107-130`) bereits vorbereitet, aber auskommentiert.

---

#### 🟡 P3 — Double JSON-Parsing in `card_db_to_response`

**Fundort:** `db/card_repo.rs:106-131`

**Problem:** `image_uris_json` wird **zweimal** geparst (Zeilen 118 + 125) — einmal für `small`, einmal für `large`. Ebenso werden `legalities` und `prices` mehrfach durch die Closures `get_price`/`get_legality` geparst.

```rust
// Erster Parse
let image_url_small = serde_json::from_str::<serde_json::Value>(&card.image_uris_json)
    .ok().and_then(|v| v.get("small").and_then(|s| s.as_str()).map(String::from));
// Zweiter Parse (identischer Source-String)
let image_url_large = serde_json::from_str::<serde_json::Value>(&card.image_uris_json)
    .ok().and_then(|v| v.get("large").and_then(|s| s.as_str()).map(String::from));
```

---

#### 🟡 P4 — LIMIT via String-Formatting statt Parameter

**Fundort:** `db/card_repo.rs:80-82`

```rust
if let Some(limit) = limit {
    query.push_str(&format!(" LIMIT {}", limit));  // String-Concatenation
}
```

**Problem:** Zwar typsicher (usize), aber verhindert SQL-Query-Plan-Caching und ist ein Anti-Pattern für spätere Erweiterungen.

---

#### 🟢 P5 — colors/color_identity als JSON statt String

**Fundort:** `db/card_repo.rs:41-42`, `models.rs:19-20`

```rust
colors: serde_json::from_str(&row.get::<_, String>("colors")?).unwrap_or_default(),
```

**Problem:** Farben sind immer 1-2 Zeichen (`"W"`, `"UB"`, etc.), werden aber als JSON-Array gespeichert und bei jedem Lesen geparst. Ein einfaches `TEXT` wie `"WU"` wäre performanter und einfacher zu indexieren/filtern.

---

#### 🟢 P6 — Kein Connection-Pooling

**Fundort:** `main.rs:17-18`

```rust
struct AppState {
    db: Mutex<rusqlite::Connection>,  // Single Connection
}
```

**Problem:** WAL-Modus ist aktiviert, aber nur eine einzige Connection. Bei asynchronen Scryfall-API-Calls ist die DB für andere Befehle blockiert. Ein Pool (z. B. `r2d2`) würde Parallellesen erlauben.

---

### 1.2 Error Handling

#### 🟠 E1 — `unwraps` auf Mutex in ScryfallClient

**Fundort:** `scryfall/client.rs:41, 44, 64, 91`

```rust
let mut cache = self.cache.lock().unwrap();  // Panic bei poisoned Mutex
```

**Problem:** 4x `.unwrap()` auf einem `Mutex<LruCache>`. Sollte `AppError` via `map_err` verwenden oder zumindest mit `.unwrap_or_else(|e| ...)` behandeln.

---

#### 🟠 E2 — HTTP-Status-Codes gehen verloren

**Fundort:** `scryfall/client.rs:53-58, 81-86, 114-121`

```rust
if !response.status().is_success() {
    return Err(AppError::Unknown(format!(
        "Scryfall API error {} for card '{}'",
        response.status(), id
    )));
}
```

**Problem:** Alle HTTP-Fehler landen als `AppError::Unknown`. Keine Unterscheidung zwischen:
- **404** → Not Found (sollte behandelt werden)
- **429** → Rate Limited (Retry nötig)
- **422** → Ungültige Query (sollte an Frontend)
- **500** → Serverfehler

---

#### 🟠 E3 — DB-Path-Error wird in falschen Typ gemapped

**Fundort:** `db/connection.rs:15, 18`

```rust
.map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
```

**Problem:** Ein Tauri-Pfad-Fehler (z. B. fehlender App-Data-Ordner) wird als `ToSqlConversionFailure` klassifiziert — semantisch falsch. Sollte direkt als `AppError::Io` oder `AppError::Unknown` propagiert werden.

---

#### 🟡 E4 — Silent Error Swallowing

**Fundort:** `services/card_service.rs:76`
```rust
let _ = card_repo::insert_card(db, &card_db);  // Fehler stumm ignoriert
```

**Fundort:** `services/card_service.rs:46-51`
```rust
if let Err(e) = db.execute(...) {
    tracing::warn!("Failed to cache set info: {}", e);
}
```

**Problem:** Im ersten Fall wird der Insert-Fehler komplett ignoriert. Die Karte wird im Memory-Cache gehalten, aber nicht in SQLite persistiert — das Frontend bekommt inkonsistente Daten.

---

#### 🟡 E5 — JSON-Parse-Fehler werden stumm zu Defaults

**Fundort:** `db/card_repo.rs:107-116`

```rust
let colors: Vec<String> = serde_json::from_value(card.colors.clone()).unwrap_or_default();
let legalities: serde_json::Value = serde_json::from_str(&card.legalities).unwrap_or_else(|_| ...);
```

**Problem:** Wenn die DB-Daten korrupt sind (z. B. durch Abbruch eines Schreibvorgangs), werden leere Defaults zurückgegeben, ohne dass der Benutzer es merkt. Ein `tracing::warn!` wäre angebracht.

---

#### 🟢 E6 — Löschlogik in `remove_card_from_deck` mit redundanten Statements

**Fundort:** `db/deck_repo.rs:131-153`

```rust
// Statement 1: Decrement
conn.execute("UPDATE deck_cards SET quantity = quantity - 1 WHERE ... AND quantity > 1", ...)?;
// Statement 2: Delete (nie erreicht, da Statement 1 nur bei qty > 1 läuft)
conn.execute("DELETE FROM deck_cards WHERE ... AND quantity <= 0", ...)?;
// Statement 3: Direct Delete (redundant — Statement 1 und 2 erledigen alles)
conn.execute("DELETE FROM deck_cards WHERE deck_id = ?1 AND card_id = ?2", ...)?;
```

**Problem:** Der dritte `DELETE` ist komplett redundant — nach Statement 1 (decrement if > 1) ist die einzige verbleibende Situation `quantity == 1`, dann löscht Statement 2 den Eintrag. Die Logik funktioniert trotzdem korrekt, ist aber schwer lesbar und gibt 3 SQL Roundtrips statt der möglichen 2 (oder 1 mit einem geschickteren Query).

---

### 1.3 Code Quality & DRY

#### 🟠 Q1 — Duplizierte Struct-Definitionen (DRY)

**Fundort:** `commands.rs:17-32` + `models.rs:116-125`

```rust
// commands.rs
pub struct SearchCardsArgs { pub query: String }
pub struct GetCardArgs { pub id: String }
// ...
// models.rs (identisch!)
pub struct SearchCardsArgs { pub query: String }
pub struct GetCardArgs { pub id: String }
```

**Problem:** `SearchCardsArgs` und `GetCardArgs` sind in **beiden Dateien** definiert — eine klare DRY-Verletzung. Models.rs scheint die ältere Version zu sein; commands.rs hat zusätzliche Typen (`AddToCollectionArgs`, `CreateDeckArgs`, `LoadLoreArgs`).

---

#### 🟡 Q2 — `card_db_to_response` ist zu lang (~65 Zeilen)

**Fundort:** `db/card_repo.rs:106-181`

**Problem:** Die Funktion mischt:
1. JSON-Parsing (5 verschiedene Quellen)
2. Closure-Definitionen (`get_price`, `get_legality`)
3. Struct-Konstruktion (30 Felder)

Empfohlen: Aufteilung in Unterfunktionen für Image-Uris, Preise, Legalitäten.

---

#### 🟡 Q3 — `set_name` ist immer leer

**Fundort:** `db/card_repo.rs:161`

```rust
set: card.set_id.clone(),
set_name: String::new(),  // Immer leer!
```

**Problem:** Der Sets-Tabellen-Join fehlt. Die Karten haben `set_id` als Fremdschlüssel, aber beim Erstellen der Response wird der Set-Name nicht aus der DB geladen. Das Frontend erhält keine vernünftigen Set-Namen.

---

#### 🟡 Q4 — Manueller YAML-Parser statt `frontmatter`-Crate

**Fundort:** `services/lore_service.rs:23-32`

```rust
// Manuelles YAML-Parsing (fragil!)
for line in yaml_str.lines() {
    if let Some((key, value)) = line.split_once(':') {
        // ...
    }
}
```

**Problem:** Die `frontmatter`-Crate (Version 0.4) ist in `Cargo.toml:31` deklariert, wird aber nie verwendet. Stattdessen implementiert `lore_service` einen eigenen YAML-Frontmatter-Parser, der:
- Keine Multiline-Werte parst
- Keine verschachtelten Strukturen unterstützt
- Leerzeichen in Werten verlieren kann

---

#### 🟢 Q5 — Deutsche & Englische Fehlermeldungen gemischt

**Fundort:** `commands.rs:79, 98, 148, 206` (Deutsch) vs. `utils/error.rs` (Englisch)

```
"Karte '{}' nicht in der lokalen Datenbank gefunden"       (DE)
"Failed to cache set info: {}"                             (EN)
```

---

#### 🟢 Q6 — Hardcodiertes Limit für Kartensuche

**Fundort:** `commands.rs:57`

```rust
let cards = card_service::search_cards_local(&db, &args.query, 50)?;
```

**Problem:** Das Limit von 50 ist hardcodiert. Der `SearchCardsArgs`-Struct hat kein `limit`-Feld, das Frontend kann also nicht steuern, wie viele Ergebnisse es bekommt.

---

### 1.4 Missing Features

#### 🟠 M1 — Kein Migrations-Framework

**Fundort:** `db/connection.rs:44-53`

**Problem:** Die Migration liest eine einzelne SQL-Datei und führt sie aus — kein Tracking, welche Migrationen bereits gelaufen sind. Bei Schema-Änderungen in zukünftigen Versionen gibt es keine saubere Upgraderoutine. `CREATE TABLE IF NOT EXISTS` kaschiert das Problem bisher.

---

#### 🟠 M2 — FTS5-Suche ist deaktiviert

**Fundort:** `db/schema.rs:155-157`, `db/migrations/001_initial.sql:107-130`

**Problem:** Die FTS5-Virtual-Tabelle ist vorbereitet, aber auskommentiert. Aktuell wird `LIKE '%query%'` verwendet (ohne Indexnutzung). Mit FTS5 wäre die Suche um Größenordnungen schneller und könnte Ranking/Highlighting bieten.

---

#### 🟠 M3 — Keine Set-Namen-Auflösung

**Problem:** `card_db_to_response` gibt `set_name: String::new()` zurück. Der `sets`-Table-Join fehlt in allen Card-Queries. Eine Karte wie "Black Lotus" mit `set_id = "lea"` würde im Frontend als "lea" statt "Limited Edition Alpha" angezeigt.

---

#### 🟡 M4 — Kein Logging-Level-Konfiguration

**Fundort:** `main.rs:24-26`

```rust
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

**Problem:** Logging ist rein env-basiert (`RUST_LOG`). Keine Datei-Logging, keine Log-Rotation, keine Konfiguration über die App-Oberfläche. Bei Produktions-Deployment ein Problem.

---

#### 🟡 M5 — Keine Tests für Services & Commands

**Fundort:** Nur `db/connection.rs:68-124` und `db/mod.rs:12-19` haben Tests.

**Problem:** Keine Tests für:
- `card_service`, `deck_service`, `lore_service`
- `commands` (keine Integrationstests)
- `scryfall::client` (keine Mock-Tests)

---

#### 🟢 M6 — `frontmatter`-Crate deklariert aber ungenutzt

**Fundort:** `Cargo.toml:31`

```toml
frontmatter = "0.4"
```

**Problem:** Tote Abhängigkeit. Wird nirgendwo in `use`-Statements referenziert.

---

#### 🟢 M7 — Kein Event/WebSocket-System

**Problem:** Tauri v2 unterstützt Events (`app.emit()`). Aktuell gibt es keine Mechanismen, um das Frontend über asynchrone Vorgänge zu informieren (z. B. "Scryfall-Sync abgeschlossen", "Daten neu geladen").

---

## 2. Performance-Empfehlungen

### 2.1 Sofort umsetzbar (P1-P2)

| # | Maßnahme | Datei(en) | Geschätzter Gewinn |
|---|---------|-----------|-------------------|
| 1 | **JOIN-Query für Deck+Karten** — `get_deck_with_cards` soll `deck_cards LEFT JOIN cards` verwenden statt N+1 | `deck_service.rs`, `deck_repo.rs` | 10-100x bei großen Decks |
| 2 | **Batch-Query für `get_deck_mana_curve`** — `deck_cards JOIN cards` für CMC + Farben | `commands.rs:211-218`, `deck_repo.rs` | 5-50x |
| 3 | **FTS5 aktivieren** — Uncomment in schema.rs + Migration | `db/schema.rs:155-157` | 100-1000x bei Textsuche |
| 4 | **JSON einmal parsen** — `image_uris_json` in `card_db_to_response` einmal parsen, dann auf das Value-Objekt zugreifen | `db/card_repo.rs:118-131` | 2x weniger GC/Laufzeit |

### 2.2 Mittelfristig (P3-P6)

| # | Maßnahme | Datei(en) | Geschätzter Gewinn |
|---|---------|-----------|-------------------|
| 5 | **Connection-Pooling** — `r2d2` + `r2d2-sqlite` für parallele Lesezugriffe | `main.rs`, `db/connection.rs` | Parallelisierung |
| 6 | **Colors als TEXT statt JSON** — `"WU"` statt `'["W","U"]'` | `db/schema.rs`, `models.rs`, `card_repo.rs` | Weniger Parsing |
| 7 | **Prepared Statements cachen** — `conn.prepare()` wiederverwenden statt jedes Mal neu | Alle Repositories | Weniger Compile-Zeit |
| 8 | **LRU-Cache-Größe konfigurierbar** | `scryfall/client.rs` | Flexibilität |

### 2.3 Beispiel: JOIN-Query für Deck+Karten

```sql
-- Statt N+1 Queries:
SELECT d.id AS deck_id, d.name, d.format, d.description,
       d.created_at, d.updated_at,
       dc.card_id, dc.quantity, dc.position,
       c.id, c.name, c.mana_cost, c.cmc, c.type_line, c.oracle_text,
       c.colors, c.color_identity, c.keywords, c.rarity, c.set_id,
       c.image_uris, c.artist, c.legalities, c.prices,
       s.name AS set_name
FROM decks d
LEFT JOIN deck_cards dc ON dc.deck_id = d.id
LEFT JOIN cards c ON c.id = dc.card_id
LEFT JOIN sets s ON s.id = c.set_id
WHERE d.id = ?1
ORDER BY dc.position;
```

---

## 3. Sicherheits-Check

### 3.1 SQL Injection

| Query-Typ | Parameterisiert? | Risiko |
|-----------|-----------------|--------|
| `named_params!` / `params!` (alle Repositories) | ✅ Ja | Kein Risiko |
| `format!(" LIMIT {}", limit)` in `card_repo.rs:81` | ⚠️ Nein (aber `usize`) | Gering (Typ-sicher) |
| `format!("&page={}", page)` in `client.rs:112` | ✅ Externer API-Call | Kein Risiko |

**Fazit:** Keine akute SQL-Injection-Lücke. Der `format!` für LIMIT ist ein Code-Smell, aber nicht direkt ausbeutbar, da `limit` als `usize` typisiert ist.

### 3.2 Path Traversal

| Pfad | Absicherung | Risiko |
|------|------------|--------|
| `std::fs::read_to_string(path)` in `lore_service.rs:11` | ⚠️ Nur wenn Frontend Pfad steuern kann | Gering (aktuell nicht exponiert) |
| `app_data_dir().join(...)` in `connection.rs:21` | ✅ Sicher (Tauri-begrenzt) | Kein Risiko |

### 3.3 API-Keys / Secrets

- **Keine API-Keys**: Scryfall-API ist öffentlich, kein Auth-Token nötig — ✅
- **Keine Passwörter** in `main.rs` oder `.env` — ✅
- **Keine unsicheren Deserializations** (alle `#[derive(Deserialize)]` mit serde) — ✅

### 3.4 Weitere Checks

| Aspekt | Status | Hinweis |
|--------|--------|---------|
| Mutex-Poison-Risiko | ⚠️ | `scryfall/client.rs` 4x `.unwrap()` — bei Cache-Poisoning panict der ganze Thread |
| `.expect()` in Hot-Path | ⚠️ | `app.handle().path().app_data_dir()` → Panic bei fehlendem App-Data |
| Fehler-Informationen nach außen | ✅ | Nur AppError → Frontend, keine Stacktraces |
| Input-Validierung | ⚠️ | `get_card(id: String)` prüft nicht auf gültiges Format — akzeptiert alles |

---

## 4. Nächste Optimierungsschritte

### Phase 1 — Hochpriorität (sofort)

- [x] Alle 19 `.rs` Dateien gelesen & analysiert
- [ ] **FTS5 aktivieren** (`schema.rs:155-157`, `001_initial.sql:107-130`)
- [ ] **N+1 in `list_decks` beheben** — JOIN-Query statt Schleife
- [ ] **N+1 in `get_deck_mana_curve` beheben** — JOIN-Query
- [ ] **JSON-Doppelparsen in `card_db_to_response` entfernen**
- [ ] **Mutex-unwrap in `scryfall/client.rs` sichern**
- [ ] **`remove_card_from_deck` Logik vereinfachen** (2 statt 3 Statements, klarerer Algorithmus)

### Phase 2 — Code-Qualität

- [ ] **Duplizierte Structs entfernen** (`commands.rs` ↔ `models.rs`)
- [ ] **`card_db_to_response` refactoren** (in `parse_images`, `parse_prices`, `parse_legalities`)
- [ ] **`frontmatter`-Crate nutzen statt handgemachtem YAML-Parser**
- [ ] **Set-Namen auflösen** (`LEFT JOIN sets` in allen Card-Queries)
- [ ] **`set_name` im Response füllen**
- [ ] **Sprache vereinheitlichen** (alle Fehlermeldungen auf Deutsch oder Englisch)
- [ ] **Limit-Parameter für Kartensuche von Frontend steuerbar machen**
- [ ] **Dead Dependency `frontmatter` entfernen** (Oder nutzen!)

### Phase 3 — Error Handling

- [ ] **HTTP-Status-Codes differenzieren** (404→NotFound, 429→RateLimited, 422→Validation)
- [ ] **DB-Path-Error korrekt mappen** → `AppError::Io`
- [ ] **Silent Error Swallowing beheben** (Loggen statt `let _ = ...`)
- [ ] **JSON-Parse-Fehler loggen** mit `tracing::warn!` statt Silent-Default

### Phase 4 — Production-Readiness

- [ ] **Connection-Pooling** (`r2d2` oder `deadpool-sqlite`)
- [ ] **Migration-Framework** (z. B. `refinery` oder eigene Versionstabelle)
- [ ] **Logging-Konfiguration** (Datei-Logging, Log-Rotation, Config-File)
- [ ] **Tests** (Services, Commands, Scryfall-Mock)
- [ ] **Health-Check-Endpunkt** für Frontend
- [ ] **Bulk-Import/Export** (CSV/JSON für Collection & Decks)
- [ ] **Tauri-Events** für Async-Benachrichtigungen
- [ ] **Config-System** (DB-Pfad, Cache-Größe, API-Rate-Limits)

### Phase 5 — Zukunft (für v0.2+)

- [ ] **Connection-Pooling** mit `r2d2` für parallele asynchrone Zugriffe
- [ ] **LRU-Cache-Metriken** (Hit-Rate, Eviction-Count)
- [ ] **Scryfall-Rate-Limiting** (aktuell 10 Concurrent — Scryfall erlaubt 10 req/s, kein Retry bei 429)
- [ ] **Full-Text-Search** mit Ranking & Highlighting (FTS5)
- [ ] **Asynchrone DB-Operationen** (optional)

---

## Anhang: Datei-Übersicht (19 Dateien)

| Datei | Zeilen | Funktion | Bewertung |
|-------|--------|----------|-----------|
| `main.rs` | 61 | Entry Point | ✅ Sauber, aber `Mutex<Connection>` suboptimal |
| `commands.rs` | 244 | Tauri Commands | ⚠️ N+1, duplizierte Structs, hardcodiertes Limit |
| `models.rs` | 198 | Data Models | ⚠️ Enthält duplizierte Input-Args |
| `db/mod.rs` | 19 | Module Index | ✅ OK |
| `db/connection.rs` | 125 | DB Init | ⚠️ Falscher Error-Type, keine Migrations-Versionierung |
| `db/schema.rs` | 171 | Schema | ⚠️ FTS5 auskommentiert |
| `db/card_repo.rs` | 182 | Card CRUD | ⚠️ LIKE-Wildcard, JSON-Doppelparsen, Format-LIMIT |
| `db/collection_repo.rs` | 107 | Collection CRUD | ✅ Sauber |
| `db/deck_repo.rs` | 193 | Deck CRUD | ⚠️ Redundante Löschlogik |
| `db/lore_repo.rs` | 135 | Lore CRUD | ✅ Sauber |
| `services/mod.rs` | 7 | Module Index | ✅ OK |
| `services/card_service.rs` | 157 | Card Logic | ⚠️ Silent Error Swallowing |
| `services/deck_service.rs` | 113 | Deck Logic | ⚠️ Wird von N+1 aufgerufen |
| `services/lore_service.rs` | 125 | Lore Logic | ⚠️ Manueller YAML-Parser |
| `scryfall/mod.rs` | 3 | Module Index | ✅ OK |
| `scryfall/models.rs` | 196 | API Models | ✅ Sauber |
| `scryfall/client.rs` | 158 | HTTP Client | ⚠️ 4x Mutex-unwrap, undifferenzierte HTTP-Fehler |
| `utils/mod.rs` | 4 | Module Index | ✅ OK |
| `utils/error.rs` | 66 | Error Types | ✅ Gut (thiserror, From-Impls) |
| **Gesamt** | **~2.200** | | |

---

## Legende

| Symbol | Bedeutung |
|--------|-----------|
| 🟠 P1/P2 | Performance — kritisch / wichtig |
| 🟡 P3 | Performance — nice-to-have |
| 🟢 P4+ | Performance — für Zukunft |
| 🟠 E1/E2 | Error Handling — kritisch |
| 🟡 E3+ | Error Handling — wichtig |
| 🟠 Q1 | Code Quality — DRY-Verstoß |
| 🟡 Q2+ | Code Quality — Verbesserung |
| 🟠 M1+ | Missing Feature — wichtig |
| 🟢 M6+ | Missing Feature — nice-to-have |
