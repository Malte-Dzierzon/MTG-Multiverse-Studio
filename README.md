# MTG Multiverse Studio — TUI

Lokales Terminal-Dashboard für Magic: The Gathering in **Rust** (`mtg-tui`, v0.3.0):
Kartensuche mit live gestreamten Kartenbildern (Kitty Graphics Protocol),
Deck-Library mit Cover-Vorschau und Manakurve, plus KI-Agent-Chat (OpenCode Zen).
Keine Cloud, kein Tracking — nur Scryfall-Bilder und optionale LLM-Anfragen.

## Stand (grob)

- **Umfang:** ~4.100 Zeilen Rust in 8 Modulen (`src/`), `ratatui` + `crossterm` fürs TUI,
  `rusqlite` (bundled) für Suche/Decks, `ureq` für Scryfall-Streaming + Agent-API.
- **Modi:** Home (Deck-Library) · Deck View (Kartentabelle) · Search (FTS über ~38k Karten) ·
  Card View (Bild + Preise, Rarity, Set, Legalities) · Import (Text/Arena/CSV) · Agent-Chat-Seitenleiste.
- **Bilder:** werden live von Scryfall geladen (Memory-LRU, nichts auf Platte).
  Ohne Kitty-Terminal läuft die App ohne Bilder (Graceful Degradation).
- **Theme:** Auto-Theme per OSC-Query (Dark/Light + Terminal-Palette).
- **Agent:** OpenCode Zen (OpenAI-kompatibel), Key aus `MTG_LLM_API_KEY` → `.env` →
  `~/.local/share/opencode/auth.json`, Settings-Screen für Runtime-Key, Skills/Pins/Notizen
  in `.mtg-tui.json` (`MTG_TUI_STORE` überschreibt den Pfad).
- **Daten:** `assets/mtg_multiverse_studio.db` (~115 MB, Scryfall-Snapshot) liegt **lokal** und ist
  per `.gitignore` ausgenommen (`assets/*.db*`) — nicht im Repo, muss separat bezogen werden.

## Setup

```bash
./run.sh                        # release-build + start (Standard-DB unter assets/)
./run.sh --db /pfad/zur/db      # eigene Datenbank
mtg-tui --help                  # Hilfe
```

Voraussetzungen: Rust (`rustup`), Datenbankdatei unter `assets/` (s. oben),
optional [Kitty](https://sw.kovidgoyal.net/kitty/) für Kartenbilder,
optional API-Key für den Agent-Chat.

## Tasten (grob)

| Kontext   | Taste                                                              |
|-----------|--------------------------------------------------------------------|
| Global    | `q` quit · `/` Suche · `g` Agent-Fokus · `Tab` Fokus wechseln · `Ctrl+D` quit |
| Home      | `j/k` Deck wählen · `Enter` öffnen · `n` neu · `i` Import · `e` Export · `x` löschen |
| Deck View | `j/k` Karte · `Enter` Details · `a` +1 ins Deck · `d` entfernen     |
| Card View | `a` ins aktive Deck                                                |
| Import    | Liste einfügen · `Ctrl+S` übernehmen · `Esc` abbrechen             |
| Agent     | Tippen + `Enter` senden (Fokus `Agent`), Key in Settings hinterlegbar |

Details können sich ändern — maßgeblich ist `src/app.rs` (State/Keybindings).

## Architektur

```
src/
├── main.rs    # CLI (--db/--help), Event-Loop, Bild-Placements
├── app.rs     # State-Maschine, Keybindings, Tests (~1.100 Zeilen)
├── ui.rs      # ratatui-Layouts (theme-getrieben, ~1.100 Zeilen)
├── db.rs      # rusqlite: Suche, Decks, Import/Export
├── agent.rs   # OpenCode-Zen-Harness, Chat-Protokoll, Store (~470 Zeilen)
├── images.rs  # Loader-Thread: Scryfall → JPEG → RGB (Memory-LRU)
├── kitty.rs   # Kitty-Graphics-Protocol: transmit/place/delete
└── theme.rs   # OSC 10/11/4 Auto-Theme (Terminal-Farben)
run.sh         # build + start (löst Symlinks auf, globaler Befehl tauglich)
assets/        # lokale DB (nicht im Repo, s. .gitignore)
```

## Rechtliches

Fan-Projekt. Keine Verbindung zu Wizards of the Coast LLC.
[Wizards Fan Policy](https://www.wizards.com/legal/wizards-fan-policy).
