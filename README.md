
## Quickstart

```bash
git clone https://github.com/Malte-Dzierzon/MTG-Multiverse-Studio.git
cd mtg-multiverse-studio
./run.sh
```

`run.sh` builds and starts the TUI. Requires Rust (`rustup`).

Optional: place a Scryfall SQLite dump at `assets/mtg_multiverse_studio.db`
or use `./run.sh --db /path/to/db`.

Card images need a Kitty-compatible terminal. Everything else works without it.



## What this is

A **learning prototype** — rough, incomplete, built to practice Rust
and Theory-Design thinking. Not a polished product. Expect bugs.



## Features (so far)

- **Deck Library** — create/delete, cover art, mana curve, value
- **Deck View** — card table, live image under cursor (Kitty graphics)
- **Search** — FTS over ~38k cards, exact-first, suggestions
- **Card View** — large image, prices, rarity, set, legalities
- **Import** — paste lists (text/Arena/CSV), `Ctrl+S` to commit
- **Agent Chat** — OpenCode Zen sidebar, runtime API key, persisted notes
- **Auto-Theme** — reads terminal colors via OSC 10/11/4
- **Graceful Degradation** — no Kitty = no images, rest works

Images stream from Scryfall into memory (LRU) — nothing hits disk.



## Stack

`ratatui`  `crossterm`  `rusqlite(bundled)`  `ureq(json,tls)`  `jpeg-decoder`  `serde_json`  `base64`  `libc`

Zero ML deps, no GPU, runs on modest hardware.



## Status

**Early prototype.** Core loops work; edges are rough.
Use at your own risk — mainly exists so I can learn by building.



## License

MIT
