# MTG Multiverse Studio

[![GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-native-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-ff8400.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18+-61dafb.svg?logo=react)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-native-blue.svg)](https://www.typescriptlang.org/)

---

## Grundidee

Eine Desktop-Anwendung die deine gesamte Magic-Kollektion lokal verwaltet — von der Kartendatenbank ueber den Deckbau bis zum interaktiven Artbook und Lore-Narrativ. Keine Cloud. Kein Tracking. Alles bleibt auf deinem Rechner, alle Daten sind offline verfuegbar.

---

## Features im Ueberblick

| Seite              | Was sie leistet                                      |
|--------------------|------------------------------------------------------|
| **Sammlung**       | Digitale Kartensammlung mit Scryfall-Anbindung und Mengenverfolgung        |
| **Deck-Labor**     | Drag-and-Drop Deck-Editor, Mana-Kurve-Charting und Wahrscheinlichkeitsberechnungen |
| **Lore-Atlas**     | Interaktiver Artbook + Storytelling-Viewer direkt verknueft mit Kartendaten (Karte -> Plane-Artwork -> Hintergrundgeschichte) |

---

## Tech-Stack

| Ebene    | Technologie                    | Zweck                          |
|----------|--------------------------------|--------------------------------|
| Framework| [Tauri v2](https://tauri.app/)  | Native Desktop-Shell (~5MB)    |
| Frontend | React + TypeScript             | Komponenten-basierter UI       |
| Build    | Vite                           | Ultra-schneller HMR & Bundling |
| Styling  | Tailwind CSS                   | Utility-first + Dark/Light     |
| State    | Zustand                        | TS-native, kein Redux           |
| Backend  | Rust (native in Tauri)         | SQLite, HTTP, KI-Inferenz      |

---

## Projektstruktur

```
mtg-multiverse-studio/
├── assets/              # Medien & Rohdaten (Bilder, Lore-Texte)
├── Documentation/       # ROADMAP.md, AGENT.md, etc.
├── src/                 # Frontend: React + TypeScript
│   ├── components/      # UI-Komponenten nach Modul sortiert
│   ├── hooks/           # Custom Hooks
│   ├── store/           # Zustand-State
│   ├── pages/           # Route-Komponenten
│   └── services/        # API-Clients (Scryfall, DB)
├── src-tauri/           # Backend: Rust + Native-Anbindung
│   ├── resources/       # LLM-Modelle & Prompt-Vorlagen
│   └── src/             # db/, ki/, scryfall/
├── package.json         # Frontend Dependencies & Scripts
├── tsconfig.json        # TypeScript-Konfiguration
└── vite.config.ts       # Vite Build-Tool (HMR, Port 1420)
```

---

## Entwickler-Leitfaden

### Voraussetzungen

| Tool       | Version                              |
|------------|--------------------------------------|
| Rust       | 1.70+ (`rustup install stable`)      |
| Node.js    | LTS (20.x) via nvm                   |
| Tauri CLI v2 | `npm install -g @tauri-apps/cli@latest` |
| Git        | 2.30+                                |

### Befehle

```bash
npm install              # Frontend Dependencies installieren
cargo build --release    # Backend kompilieren (Release-Build)
npm run tauri dev        # Live: Frontend + Backend gleichzeitig
npx tauri build          # Vollstaendiger Desktop-Build (exe/dmg/AppImage)
```

### Architektur

Frontend kommuniziert **NIEMALS** direkt mit dem Internet. Alle Datenflüsse laufen über die Tauri-Bridge:

```
┌──────────────┐      ┌─────────────────────┐      ┌──────────────┐
│  Frontend    │      │   Rust Backend      │      │ SQLite DB    │
│  (React/TS)  ├─────►│  Tauri Bridge + Cmd ├────► │  (lokal)     │
└──────────────┘      └─────────────────────┘      └──────────────┘
                           │
                           ├──► Scryfall API (on demand)
                           │
                           └──► LLM Inferenz (GGUF, On-Device)
```

---

## Rechtliches

Dieses Projekt ist ein Fan-Projekt unter der GPL-v3-Lizenz. Es steht in keiner Verbindung zu Wizards of the Coast LLC oder den Inhabern der "Magic: The Gathering"-Markenrechte. Unterliegt der [Wizards Fan Policy](https://www.wizards.com/legal/wizards-fan-policy).
