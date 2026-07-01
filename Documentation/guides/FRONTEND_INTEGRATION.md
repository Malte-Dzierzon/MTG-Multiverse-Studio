# Backend → React Frontend Integration Guide

> **Wie die Rust-Commands im React-Frontend aufgerufen werden**
> Stand: Juli 2026 — Tauri v2

---

## 1. Grundprinzip: Tauri invoke()

Der React-Frontend ruft Rust-Funktionen über die **Tauri Bridge** auf.  
Das Frontend hat **NIEMALS** direkten Internetzugriff — alle Daten fließen durchs Rust-Backend.

```typescript
// TypeScript → Tauri IPC → Rust → SQLite/Scryfall → zurück
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('search_cards', { 
  args: { query: "Black Lotus" } 
});
```

---

## 2. Verfügbare Commands (8 Stück)

| Command | Input | Output | Beschreibung |
|---------|-------|--------|-------------|
| `search_cards` | `{ query: string }` | `SearchResult` | Karten lokal suchen |
| `get_card` | `{ id: string }` | `CardResponse` | Einzelkarte abrufen |
| `add_to_collection` | `{ card_id, quantity?, condition? }` | `CollectionItemResponse` | Karte zur Sammlung |
| `create_deck` | `{ name, format?, description? }` | `CreatedDeckResponse` | Neues Deck |
| `get_deck` | `{ id: string }` | `DeckResponse` | Deck + Karten |
| `list_decks` | `{}` | `DeckResponse[]` | Alle Decks |
| `load_lore_entries` | `{ lore_type? }` | `LoreEntryResponse[]` | Lore laden |
| `get_deck_mana_curve` | `{ id: string }` | `{ mana_curve, color_balance }` | Deck-Analyse |

---

## 3. TypeScript API-Client (src/services/api.ts)

So sieht der vollständige Client aus, der alle Commands wrappt:

```typescript
// src/services/api.ts — Generierter API-Client

import { invoke } from '@tauri-apps/api/core';

// ─── Types ─────────────────────────────────

export interface CardResponse {
  id: string;
  name: string;
  mana_cost?: string;
  cmc: number;
  type_line: string;
  card_text?: string;
  colors: string[];
  color_identity: string[];
  keywords: string[];
  rarity: string;
  set: string;
  set_name: string;
  artist?: string;
  image_url_small?: string;
  image_url_large?: string;
  prices?: {
    usd?: string;
    usd_foil?: string;
    eur?: string;
    tix?: string;
  };
  legalities?: {
    standard: string;
    modern: string;
    legacy: string;
    vintage: string;
    commander: string;
    pioneer: string;
    pauper: string;
  };
}

export interface SearchResult {
  cards: CardResponse[];
  total: number;
  from_cache: boolean;
}

export interface CollectionItem {
  id: number;
  card: CardResponse;
  quantity: number;
  condition: string;
  notes?: string;
  added_at: string;
}

export interface DeckResponse {
  id: number;
  name: string;
  format?: string;
  description?: string;
  created_at: string;
  updated_at?: string;
  cards: DeckCardResponse[];
}

export interface DeckCardResponse {
  card: CardResponse;
  quantity: number;
  position: number;
}

export interface LoreEntry {
  id: number;
  title: string;
  lore_type: string;
  content: string;
  metadata?: any;
  related_cards: string[];
}

export interface ManaCurve {
  mana_curve: Record<string, number>;
  color_balance: {
    white: number;
    blue: number;
    black: number;
    red: number;
    green: number;
    colorless: number;
  };
}

// ─── API Functions ─────────────────────────

export async function searchCards(query: string): Promise<SearchResult> {
  return invoke('search_cards', { args: { query } });
}

export async function getCard(id: string): Promise<CardResponse> {
  return invoke('get_card', { args: { id } });
}

export async function addToCollection(
  cardId: string, 
  quantity?: number, 
  condition?: string
): Promise<CollectionItem> {
  return invoke('add_to_collection', { 
    args: { card_id: cardId, quantity, condition } 
  });
}

export async function createDeck(
  name: string, 
  format?: string, 
  description?: string
): Promise<{ deck: DeckResponse }> {
  return invoke('create_deck', { 
    args: { name, format, description } 
  });
}

export async function getDeck(id: number): Promise<DeckResponse> {
  return invoke('get_deck', { args: { id: String(id) } });
}

export async function listDecks(): Promise<DeckResponse[]> {
  return invoke('list_decks');
}

export async function loadLoreEntries(loreType?: string): Promise<LoreEntry[]> {
  return invoke('load_lore_entries', { args: { lore_type: loreType } });
}

export async function getDeckManaCurve(id: number): Promise<ManaCurve> {
  return invoke('get_deck_mana_curve', { args: { id: String(id) } });
}
```

---

## 4. React Hooks (src/hooks/)

Hooks wrappen die API-Funktionen mit React-Query oder eigenem State:

```typescript
// src/hooks/useCards.ts
import { useState, useEffect } from 'react';
import { searchCards, CardResponse } from '../services/api';

export function useCardSearch(query: string) {
  const [results, setResults] = useState<CardResponse[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      return;
    }
    
    setLoading(true);
    searchCards(query)
      .then(res => setResults(res.cards))
      .catch(err => setError(err.toString()))
      .finally(() => setLoading(false));
  }, [query]);

  return { results, loading, error };
}

// src/hooks/useDeck.ts
export function useDeck(id: number) { ... }
// Analog für alle anderen Endpunkte
```

---

## 5. React Pages — Struktur

```
src/pages/
├── collection/
│   └── index.tsx         ← CollectionView (Kartenliste + Suche + Filter)
├── deckbuilder/
│   └── index.tsx         ← DeckEditor (Drag&Drop, Mana-Kurve, Analyse)
└── lore_atlas/
    └── index.tsx         ← LoreView (Artbook + Story-Reader)
```

### Beispiel: Minimal Collection View

```tsx
// src/pages/collection/index.tsx
import { useState } from 'react';
import { useCardSearch } from '../../hooks/useCards';

export default function CollectionView() {
  const [query, setQuery] = useState('');
  const { results, loading } = useCardSearch(query);

  return (
    <div className="p-4">
      <h1 className="text-2xl font-bold">Sammlung</h1>
      <input
        type="text"
        placeholder="Karte suchen..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        className="w-full p-2 border rounded mt-2"
      />
      {loading && <p>Lade...</p>}
      <div className="grid grid-cols-4 gap-4 mt-4">
        {results.map(card => (
          <div key={card.id} className="border rounded p-2">
            {card.image_url_small && (
              <img src={card.image_url_small} alt={card.name} />
            )}
            <p className="font-bold mt-1">{card.name}</p>
            <p className="text-sm">{card.type_line}</p>
            <p className="text-xs">{card.set_name}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
```

---

## 6. State-Management mit Zustand

```typescript
// src/store/collectionStore.ts
import { create } from 'zustand';
import { CardResponse, DeckResponse } from '../services/api';

interface AppState {
  // Collection
  searchQuery: string;
  searchResults: CardResponse[];
  setSearchQuery: (q: string) => void;
  setSearchResults: (cards: CardResponse[]) => void;
  
  // Decks
  decks: DeckResponse[];
  activeDeck: DeckResponse | null;
  setDecks: (decks: DeckResponse[]) => void;
  setActiveDeck: (deck: DeckResponse | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  searchQuery: '',
  searchResults: [],
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSearchResults: (cards) => set({ searchResults: cards }),
  
  decks: [],
  activeDeck: null,
  setDecks: (decks) => set({ decks }),
  setActiveDeck: (deck) => set({ activeDeck: deck }),
}));
```

---

## 7. Tauri v2 Permissions (für Capabilities)

In Tauri v2 gibt es keine bar `allowlist` mehr. Stattdessen werden **Capabilities** verwendet:

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:app:default"
  ]
}
```

Für die Rust-Seite braucht man **keine** speziellen Berechtigungen, weil:
- HTTP-Requests (Scryfall API) → **Rust macht sie direkt**, nicht über die WebView
- SQLite → **Rust hat direct filesystem access**
- Markdown lesen → **Rust liest mit `std::fs`**

Die Permissions sind nur wichtig, wenn das **Frontend** über die Tauri-API aufs Filesystem zugreifen will (z.B. Datei-Dialoge).

---

## 8. Datenfluss-Diagramm (vollständig)

```
React Component
    │
    ├─ useEffect / Button Click
    │    │
    │    ▼
    ├─ src/hooks/useCards.ts
    │    │
    │    ▼
    ├─ src/services/api.ts
    │    │  invoke('search_cards', { args: { query } })
    │    │
    │    ▼  ───────── Tauri IPC Bridge ─────────
    │
    Rust src/commands.rs
    │  #[tauri::command]
    │  fn search_cards(app, args) -> Result<SearchResult>
    │    │
    │    ▼
    │  AppState → db (Mutex geschützt)
    │    │
    │    ├─ 1. SQLite fragen (card_repo::search_cards_by_name)
    │    │     └─ Gefunden? → return ✅
    │    │
    │    └─ 2. Scryfall API (async, via card_service)
    │          └─ in SQLite speichern → return
    │    │
    │    ▼  ───────── Tauri IPC Bridge ─────────
    │
    TypeScript erhält SearchResult { cards, total }
    │
    ▼
React rendert CardPreview-Komponenten
```
