# MTG Multiverse Studio — TUI

Ein lokales Terminal-Dashboard für Magic: The Gathering in **Rust**:
Kartensuche mit live gestreamten Kartenbildern (Kitty Graphics Protocol),
Deck-Library mit Cover-Vorschau und Manakurve. Keine Cloud, kein Tracking.

## Features

| Modus            | Was er leistet                                                              |
|------------------|-----------------------------------------------------------------------------|
| **Deck Library** | Decks anlegen/löschen, Cover-Bild der stärksten Karte, Manakurve, Wert      |
| **Deck View**    | Kartentabelle mit gleitender Auswahl, Live-Kartenbild unter dem Cursor      |
| **Search**       | FTS-Suche über 38k Karten, exakte Treffer zuerst, Vorschlagsliste           |
| **Card View**    | Großes Bild + Preise, Rarity, Set, Legalities                               |
| **Import**       | Decklisten einfügen (Text/Arena/CSV), `Ctrl+S` importiert                   |
| **Auto-Theme**   | Farben werden per OSC-Query vom Terminal gelesen (Dark/Light + Palette)     |

Bilder werden **live von Scryfall gestreamt** — nur Memory-Cache, nichts landet auf der Platte.
Ohne Kitty-Terminal läuft die App komplett ohne Bilder (Graceful Degradation).

## Setup

```bash
./run.sh                        # build + start
./run.sh --db /pfad/zur/db     # eigene Datenbank
```

Voraussetzungen: Rust (`rustup`), [Kitty](https://sw.kovidgoyal.net/kitty/) für Kartenbilder.

## Tasten

| Kontext       | Taste                                              |
|---------------|----------------------------------------------------|
| Global        | `q` quit · `/` Suche · `g` Agent (bald) · `Ctrl+D` quit |
| Home          | `j/k` Deck wählen · `Enter` öffnen · `n` neu · `i` Import · `e` Export · `x` löschen |
| Deck View     | `j/k` Karte · `Enter` Details · `a` +1 ins Deck · `d` entfernen |
| Card View     | `a` ins aktive Deck                                |
| Import        | Liste einfügen · `Ctrl+S` übernehmen · `Esc` abbrechen |

## Architektur

```
src/
├── main.rs    # Event-Loop, Bild-Placements
├── app.rs     # State-Maschine, Keybindings, Animation
├── ui.rs      # ratatui-Layouts (transparent, theme-getrieben)
├── db.rs      # rusqlite: Suche, Decks, Import/Export
├── images.rs  # Loader-Thread: Scryfall → JPEG → RGB (Memory-LRU)
├── kitty.rs   # Kitty-Graphics-Protocol: transmit/place/delete
└── theme.rs   # OSC 10/11/4 Auto-Theme (Terminal-Farben)
```

Der Agent-Harness (OpenCode Zen, OpenAI-kompatibel) folgt als nächster Schritt.

## Rechtliches

Fan-Projekt. Keine Verbindung zu Wizards of the Coast LLC.
[Wizards Fan Policy](https://www.wizards.com/legal/wizards-fan-policy).
