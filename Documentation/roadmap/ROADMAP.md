# MTG Multiverse Studio — Roadmap & To-Do Liste

> **Aktueller Stand:** Phase 1 (Backend) abgeschlossen.  
> **Letztes Update:** Juli 2026

---

## ✅ Phase 0: Projektsetup — ABGESCHLOSSEN

- [x] Grundlegende Ordnerstruktur erstellt
- [x] Config-Dateien angelegt (`package.json`, `Cargo.toml`, `tsconfig.json`, etc.)
- [x] Dokumentationen geschrieben (Roadmap, Umsetzungsmöglichkeiten, Feature-Research)
- [x] Story/Lore Datenstruktur konzipiert & dokumentiert
- [x] `.gitignore` erstellt (node_modules, dist, target, IDE, OS)
- [x] README.md im Root aktualisiert
- [x] Leere Stub-Dateien entfernt (config.rs, repository.rs, ki/*.rs)
- [x] Icons für Tauri-Build erstellt (32x32, 128x128, 256x256)

---

## ✅ Phase 1: Backend-Grundgerüst (Rust) — ABGESCHLOSSEN

### Datenbank-Schema
- [x] SQL-Migration (001_initial.sql) mit 6 Tabellen
- [x] `cards` — Kartendaten (Scryfall-ID, Name, Mana-Kosten, Typ, Set, Oracle-Text, Legalität, Preise, Bilder, etc.)
- [x] `sets` — MTG Sets mit Metadaten
- [x] `collection` — User-Sammlung (Card-ID, Menge, Zustand, Notizen)
- [x] `decks` — Deck-Metadaten (Name, Format, Beschreibung)
- [x] `deck_cards` — Many-to-Many: Deck ↔ Card + Menge + Position
- [x] `lore_entries` — Story-Einträge (Titel, Typ, Markdown-Pfad, YAML-Frontmatter, verknüpfte Karten)

### Tauri Commands (8 Stück)
- [x] `search_cards(query)` → Karten lokal suchen (via Service)
- [x] `get_card(id)` → Einzelkarte aus DB
- [x] `add_to_collection(card_id, qty, condition)` → Karte zur Sammlung
- [x] `create_deck(name, format, description)` → Neues Deck
- [x] `get_deck(id)` → Deck inkl. Karten
- [x] `list_decks()` → Alle Decks
- [x] `load_lore_entries(lore_type?)` → Lore-Einträge (gefiltert)
- [x] `get_deck_mana_curve(id)` → Mana-Kurve + Farbbalance

### Repository-Layer
- [x] `card_repo.rs` — CRUD für cards (insert, get_by_id, search_by_name, count)
- [x] `collection_repo.rs` — CRUD für collection (add, get, update, remove, count)
- [x] `deck_repo.rs` — CRUD für decks + deck_cards (create, get, add_card, remove_card)
- [x] `lore_repo.rs` — CRUD für lore_entries (insert, get, update, delete)

### Services (Business Logic)
- [x] `card_service.rs` — 3-Stufen-Cache (DB → LRU → Scryfall API), Scryfall-zu-CardDb-Konvertierung
- [x] `deck_service.rs` — Mana-Kurve, Farbbalance, Deck mit Karten auflösen
- [x] `lore_service.rs` — Markdown-Parsing, YAML-Frontmatter-Extraktion aus assets/stories/

### Scryfall API Integration
- [x] HTTP-Client mit Rate-Limiting (Semaphore, max 10 concurrent)
- [x] Vollständige Modell-Strukturen (Card, Set, Prices, Legalities, ImageUris)
- [x] LRU-Cache (1000 Karten, in-memory)
- [x] Error-Handling mit AppError (Tauri-konform)

### Tests
- [x] 3 Unit-Tests für DB-Initialisierung + Tabellenstruktur
- [x] Build: `cargo check` — 0 Errors

---

## 🟡 Phase 2: Frontend-Grundgerüst (React + TypeScript) — AUSGESETZT

> **Entscheidung:** Das Frontend wird bewusst zurückgestellt.  
> Es soll später in High Quality gebaut werden — siehe `FRONTEND_INTEGRATION.md` für die Anbindungs-Doku.

### Vorbereitet (Dateien existieren als Stubs)
- [ ] Vite + React Template ✅ (von Tauri generiert)
- [ ] `src/services/api.ts` — Komplette TypeScript-API-Client bereit (muss nur eingefügt werden)
- [ ] `src/hooks/` — Custom Hooks (useCards, useDeck etc.)
- [ ] `src/store/` — Zustand Store
- [ ] Routing: `/`, `/collection`, `/deck/:id`, `/lore`
- [ ] Tailwind CSS Setup
- [ ] Komponenten: Button, CardPreview, SearchInput, Modal, LoadingSpinner

---

## 🔴 Phase 3: Scryfall Bulk-Import & Daten — NÄCHSTER SCHRITT

- [ ] **Bulk-Import-Befehl** (`cargo run -- import`)
  - [ ] Oracle Cards (~180 MB) streamen & in SQLite parsen
  - [ ] Batch-Inserts (100er-Transaktionen für Performance)
  - [ ] Fortschrittsanzeige via tracing
- [ ] **Bild-Strategie** (Hybrid: Scryfall CDN streamen + optionaler lokaler Cache)
- [ ] **Preis-Integration**
  - [ ] Cardmarket API (EU, low/avg/high/trend)
  - [ ] TCGPlayer API (US, market price)
  - [ ] Preis-Caching + Verlaufstabelle

---

## 🔵 Phase 4: Deck-Labor & Analyse

- [ ] **Goldfishing** — Starthand-Simulator
- [ ] **Format-Legality-Check** (Karte legal in Standard/Modern/Commander?)
- [ ] **Auto-Kategorisierung** (Ramp, Draw, Removal, Boardwipe…)
- [ ] **Wahrscheinlichkeitsberechnung** ("Chance auf X in Starthand?")
- [ ] **Synergie-Analyse** (KI-gestützt, später)

---

## 🟢 Phase 5: Lore-Atlas & Artbook

- [ ] Markdown-Parsing für `assets/stories/` (Backend fertig ✅)
- [ ] Artbook-Viewer
- [ ] Story-Reader mit Inline-Bildern
- [ ] Timeline/Chronologie
- [ ] Plane-Explorer

---

## 🔵 Phase 6: KI-Assistent (On-Device LLM)

- [ ] Candle.rs oder llama.cpp via FFI einbinden
- [ ] GGUF-Modelle (Llama, Mistral — quantisiert)
- [ ] Deck-Analyse & Verbesserungsvorschläge
- [ ] Lore-Fragen beantworten

---

## 🎯 Phase 7: Polish & Release

- [ ] Dark/Light Mode Toggle
- [ ] Tastaturnavigation
- [ ] Virtuelle Listen für große Sammlungen
- [ ] Desktop-Bundles (exe/AppImage/dmg)
- [ ] README mit Installationsanleitung

---

## 🔍 Technische Entscheidungen

| Entscheidung | Empfehlung | Status |
|---|---|---|
| State Management | **Zustand** | ⬜ Noch nicht entschieden |
| UI-Framework | **Shadcn/ui** + Tailwind | ⬜ Noch nicht entschieden |
| Charts | **Recharts** | ⬜ Noch nicht entschieden |
| KI-Engine | **Candle.rs** (nativer Rust) | ⬜ Noch nicht entschieden |
| Story-Datenformat | **Hybrid: MD für Edit, SQLite für Suche** | ✅ Entscheiden |
| Bilder | **Stream (CDN) + optionaler Lokal-Cache** | ✅ Entscheiden |
| Preise | **Cardmarket (EU) + TCGPlayer (US) + Scryfall-Fallback** | ✅ Entscheiden |

---

## 📐 Prioritäten (aktuell)

1. 🔴 **Bulk-Import** — Kartendaten in die DB bekommen
2. 🟡 **CLI-Test-Harness** — Backend ohne Frontend testen
3. 🟡 **Preis-Integration** — Cardmarket anbinden
4. 🔵 **Deck-Analyse** — Goldfishing, Legality-Check
5. 🟢 **Lore-Atlas** — Markdown importieren
6. 🔵 **Frontend** — High Quality UI (später)
