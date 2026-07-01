# MTG Multiverse Studio — Roadmap & To-Do Liste

## Phase 0: Projektsetup (JETZT)

### ✅ Done
- [x] Grundlegende Ordnerstruktur erstellt
- [x] Config-Dateien angelegt (`package.json`, `Cargo.toml`, `tsconfig.json`, etc.)
- [x] Dokumentation mit Umsetzungsmöglichkeiten geschrieben
- [x] Story/Lore Datenstruktur konzipiert & dokumentiert

### 🔧 TODO: Struktur bereinigen/ergänzen
- [ ] `.env.example` befüllen (Tauri Config Vars)
- [ ] `.gitignore` richtig ausfüllen (`node_modules`, `dist`, `src-tauri/target`, etc.)
- [ ] `README.md` im Root schreiben (Projektbeschreibung, Setup-Anleitung)
- [ ] `public/favicon.ico` hinzufügen
- [ ] Leere Service/Util-Dateien entfernen die nicht gebraucht werden

### 🔧 TODO: Tauri Config vervollständigen
- [ ] `tauri.conf.json`: `allowList` für Filesystem-Zugriff (Story-Markdown lesen!)
- [ ] `tauri.conf.json`: `httpAllowlist` für Scryfall API (`https://api.scryfall.com`)
- [ ] `tauri.conf.json`: Fenster-Konfiguration prüfen (Größe, Titlebar, etc.)

---

## Phase 1: Backend-Grundgerüst (Rust)

### Datenbank-Schema entwerfen
- [ ] SQL-Migration schreiben für Tabellen:
  - [ ] `cards` — Kartendaten (Scryfall-ID, Name, Mana-Kosten, Typ, Set, etc.)
  - [ ] `collection` — User's Sammlungsdaten (Card-ID, Menge, Zustand, etc.)  
  - [ ] `decks` — Deck-Namen, Beschreibung, Format
  - [ ] `deck_cards` — Verknüpfung Table: Deck ↔ Card + Anzahl
  - [ ] `lore_entries` — Story-Einträge (Titel, Typ, Inhalt als Text oder Dateipfad)

### Tauri Commands implementieren
- [ ] `get_card(id)` → Kartendetails aus DB
- [ ] `search_cards(query)` → Karten suchen (Lokal + Scryfall Fallback)
- [ ] `add_to_collection(card_id, qty)` → Karte zur Sammlung hinzufügen
- [ ] `create_deck(name, format)` → Neues Deck erstellen  
- [ ] `load_lore_entries()` → Alle Storys aus Markdown laden

### Scryfall API Integration
- [ ] HTTP-Client für Scryfall einrichten (async mit Tokio)
- [ ] Model-Strukturen definieren (`Card`, `Set`, `Price`)
- [ ] Rate-Limiting implementieren
- [ ] Lokales Caching der abgerufenen Karten

---

## Phase 2: Frontend-Grundgerüst (React + TypeScript)

### Setup & Routing
- [x] Vite + React Template
- [ ] `react-router-dom` installieren und konfigurieren
- [ ] Routing definieren: `/`, `/collection`, `/decks/:id`, `/lore`
- [ ] Globale Styles (`index.css`) mit CSS-Variablen für Dark/Light Mode

### Komponenten-Bibliothek (Common)
- [ ] `Button` — Primary, Secondary, Icon-Varianter
- [ ] `CardPreview` — Zeigt Kartenvorschau mit Bild von Scryfall  
- [ ] `Modal` — Für Details/Dialoge
- [ ] `SearchInput` — Globale Suche nach Karten
- [ ] `LoadingSpinner` / `SkeletonLoader`

### Collection View (MVP)
- [ ] Kartenliste mit Pagination
- [ ] Filter nach Set, Typ, Seltenheit
- [ ] Sortierung (Name, Datum hinzugefügt, Menge)
- [ ] Karte klicken → Detail-Ansicht im Modal

---

## Phase 3: Deck-Labor

### Deck-Editor
- [ ] Drag & Drop Karten ins Deck
- [ ] Mana-Kurve berechnen und anzeigen (Chart.js oder Recharts)  
- [ ] Farbbalance-Anzeige
- [ ] Format-Filterung (nur legale Karten für das gewählte Format)
- [ ] Deck exportieren/importieren (CSV, Scryfall-Link)

### Analyse-Tools
- [ ] Wahrscheinlichkeitsberechnung ("Chance X Karte in Starthand?")
- [ ] Mana-Kurve Chart
- [ ] Synergie-Analyse (später KI-gestützt)

---

## Phase 4: Lore-Atlas & Artbook

### Daten importieren
- [ ] Markdown-Parsing für `assets/stories/` Dateien
- [ ] YAML Frontmatter lesen → Metadaten extrahieren  
- [ ] Bilder lazy-loaden aus `assets/artbook/`

### UI-Komponenten
- [ ] **Artbook-Viewer** — Hochwertige Bildanzeige mit Zoom (wie ein echtes Artbook)
- [ ] **Story-Reader** — Markdown-rendern mit Bildern inline
- [ ] **Timeline/Chronologie** — Storys chronologisch anordnen
- [ ] **Plane-Explorer** — Plane-Übersicht mit Karten als "Wolkenatlas"

---

## Phase 5: KI-Assistent (On-Device LLM)

### Modell-Auswahl & Integration
- [ ] Candle.rs oder llama.cpp via FFI einbinden  
- [ ] GGUF-Modelle unterstützen (Llama, Mistral — quantisiert)
- [ ] Model-Pfad aus Config lesen (`config.toml` oder Umgebungsvariable)

### Features
- [ ] "Welche Karten synergisieren mit meiner Karte X?"
- [ ] Lore-Fragen beantworten ("Wer ist Gideon Jura?")  
- [ ] Deck-Analyse & Verbesserungsvorschläge

---

## Phase 6: Polish & Release

### UI/UX
- [ ] Dark Mode / Light Mode Toggle
- [ ] Responsive Layout (verschiedene Fenstergrößen)
- [ ] Ladeanimationen, Transitions  
- [ ] Tastaturnavigation

### Performance
- [ ] Datenbank-Abfragen optimieren (Indizes!)
- [ ] Bilder lazy-loaden & komprimieren (WebP)
- [ ] Virtuelle Listen für große Kartensammlungen (>500 Karten)

### Release
- [ ] Desktop-Bundles erstellen (Windows .exe, Linux .AppImage, macOS .dmg)
- [ ] Icons für alle Plattformen erstellen  
- [ ] README mit Installationsanleitung vervollständigen
- [ ] GPL-v3 Lizenz-Hinweis + Wizards Fan Policy Notice

---

## 🔍 Technische Entscheidungen die noch offen sind

| Entscheidung | Optionen | Empfehlung | Zustand |
|---|---|---|---|
| State Management | Zustand vs Redux Toolkit | **Zustand** (einfacher) | ⬜ Entscheiden |
| UI-Framework | Shadcn/ui vs Mantine vs ChakraUI | **Shadcn/ui** + Tailwind | ⬜ Entscheiden |  
| Charts | Chart.js / Recharts / Nivo | **Recharts** (React-native) | ⬜ Entscheiden |
| KI-Engine | Candle.rs vs llama.cpp FFI | **Candle.rs** (nativer Rust) | ⬜ Testen |
| Story-Datenformat | Nur Markdown vs SQLite + Markdown | **Hybrid: MD für Edit, SQLite für Suche** | ⬜ Entscheiden |

---

## 📐 Prioritäten-Ranking (wichtigste Tasks zuerst)

1. 🔴 **Tauri Config vervollständigen** (Permissions!)
2. 🔴 **Datenbank-Schema designen** (Ohne DB geht nichts)
3. 🔴 **Scryfall API Integration** (Kartendaten sind das Fundament)  
4. 🟡 **Collection View UI** (erstes sichtbares Feature)
5. 🟡 **Deck-Editor Grundgerüst**
6. 🟢 **Lore-Atlas mit Artbook-Viewer**
7. 🔵 **KI-Assistent** (komplex, kann später kommen)
