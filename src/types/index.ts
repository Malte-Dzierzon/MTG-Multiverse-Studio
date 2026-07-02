// MTG Multiverse Studio — TypeScript Types
// Generated from Rust models.rs (CardResponse, DeckResponse, etc.)

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
  // Extended fields from newer commands
  set_code?: string;
  collector_number?: string;
  layout?: string;
  card_faces?: CardFace[];
  power?: string;
  toughness?: string;
  loyalty?: string;
  produced_mana?: string[];
}

export interface CardFace {
  name: string;
  mana_cost?: string;
  type_line: string;
  oracle_text?: string;
  colors: string[];
  image_url_small?: string;
  image_url_large?: string;
  power?: string;
  toughness?: string;
}

export interface SearchResult {
  cards: CardResponse[];
  total: number;
  from_cache: boolean;
  query_time_ms?: number;
}

export interface CollectionItem {
  id: number;
  card: CardResponse;
  quantity: number;
  condition: string;
  notes?: string;
  added_at: string;
}

export interface AddToCollectionArgs {
  card_id: string;
  quantity?: number;
  condition?: string;
  notes?: string;
}

export interface CollectionItemResponse {
  item: CollectionItem;
  created: boolean;
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

export interface CreatedDeckResponse {
  deck: DeckResponse;
}

export interface CreateDeckArgs {
  name: string;
  format?: string;
  description?: string;
}

export interface DeckManaCurve {
  mana_curve: Record<string, number>;
  color_balance: {
    white: number;
    blue: number;
    black: number;
    red: number;
    green: number;
    colorless: number;
  };
  total_cards: number;
  avg_cmc: number;
}

export interface DeckGoldfishResult {
  hand: CardResponse[];
  library: number;
  turns: GoldfishTurn[];
}

export interface GoldfishTurn {
  turn: number;
  drawn: CardResponse[];
  hand: CardResponse[];
  mana_available: Record<string, number>;
  played: CardResponse[];
}

export interface DeckLegalityResult {
  legalities: {
    standard: boolean;
    modern: boolean;
    legacy: boolean;
    vintage: boolean;
    commander: boolean;
    pioneer: boolean;
    pauper: boolean;
  };
  violations: string[];
}

export interface LoreEntry {
  id: number;
  title: string;
  lore_type: string;
  content: string;
  metadata?: any;
  related_cards: string[];
}

export interface LoreEntryResponse {
  entry: LoreEntry;
}

export interface LoadLoreArgs {
  lore_type?: string;
}

export interface SetResponse {
  id: string;
  name: string;
  set_type: string;
  released_at: string;
  card_count: number;
  icon_svg_uri?: string;
}

export interface PriceResponse {
  card_id: string;
  usd?: string;
  usd_foil?: string;
  eur?: string;
  eur_foil?: string;
  tix?: string;
  source: string;
  updated_at: string;
}

export interface BulkImportResult {
  imported: number;
  updated: number;
  errors: string[];
  duration_ms: number;
}

export interface ImportStatus {
  running: boolean;
  progress: number;
  current_set?: string;
  total_cards: number;
  imported_cards: number;
  errors: string[];
}

// ==========================================================================
// UI State Types
// ==========================================================================

export type PageRoute = 
  | '/' 
  | '/collection' 
  | '/deckbuilder' 
  | '/deck/:id'
  | '/lore' 
  | '/lore/:id'
  | '/settings';

export type TabId = 'collection' | 'deckbuilder' | 'lore' | 'hub' | 'settings';

export interface TabConfig {
  id: TabId;
  label: string;
  icon: string;
  route: string;
  color: string;
}

export const TABS: TabConfig[] = [
  { id: 'hub', label: 'Portal', icon: 'sparkles', route: '/', color: 'parchment' },
  { id: 'collection', label: 'Sammlung', icon: 'archive', route: '/collection', color: 'leather' },
  { id: 'deckbuilder', label: 'Deck-Labor', icon: 'flask-conical', route: '/deckbuilder', color: 'crimson' },
  { id: 'lore', label: 'Lore-Atlas', icon: 'book-open', route: '/lore', color: 'wood' },
  { id: 'settings', label: 'Einstellungen', icon: 'settings', route: '/settings', color: 'muted' },
];

// Mana color utilities
export const MANA_COLORS = {
  W: { name: 'Weiß', symbol: '☀', bg: '#f5f2ed', text: '#262425' },
  U: { name: 'Blau', symbol: '💧', bg: '#1e90ff', text: '#fff' },
  B: { name: 'Schwarz', symbol: '☠', bg: '#262425', text: '#f5f2ed' },
  R: { name: 'Rot', symbol: '🔥', bg: '#733030', text: '#f5f2ed' },
  G: { name: 'Grün', symbol: '🌿', bg: '#2d5a2d', text: '#f5f2ed' },
  C: { name: 'Farblos', symbol: '◆', bg: '#8c837e', text: '#161515' },
} as const;

export type ManaSymbol = keyof typeof MANA_COLORS;

export function getManaColor(symbol: string): { bg: string; text: string; name: string } {
  const upper = symbol.toUpperCase() as ManaSymbol;
  return MANA_COLORS[upper] || { bg: '#8c837e', text: '#161515', name: symbol };
}

// Condition labels
export const CONDITIONS = [
  { value: 'mint', label: 'Mint' },
  { value: 'nm', label: 'Near Mint' },
  { value: 'ex', label: 'Excellent' },
  { value: 'gd', label: 'Good' },
  { value: 'lp', label: 'Light Played' },
  { value: 'pl', label: 'Played' },
  { value: 'po', label: 'Poor' },
] as const;

export type Condition = typeof CONDITIONS[number]['value'];

// Rarity order
export const RARITY_ORDER = ['common', 'uncommon', 'rare', 'mythic', 'special', 'bonus'] as const;
export type Rarity = typeof RARITY_ORDER[number];

export function rarityRank(rarity: string): number {
  const idx = RARITY_ORDER.indexOf(rarity.toLowerCase() as Rarity);
  return idx >= 0 ? idx : 99;
}