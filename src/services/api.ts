// MTG Multiverse Studio — Tauri API Client
// Wraps all invoke() calls to Rust backend with proper TypeScript types

import { invoke } from '@tauri-apps/api/core';
import type {
  CardResponse,
  SearchResult,
  CollectionItem,
  CollectionItemResponse,
  AddToCollectionArgs,
  DeckResponse,
  DeckCardResponse,
  CreatedDeckResponse,
  CreateDeckArgs,
  DeckManaCurve,
  DeckGoldfishResult,
  DeckLegalityResult,
  LoreEntry,
  LoreEntryResponse,
  LoadLoreArgs,
  SetResponse,
  PriceResponse,
  BulkImportResult,
  ImportStatus,
} from '../types';

// ============================================================================
// CARD OPERATIONS
// ============================================================================

export async function searchCards(query: string): Promise<SearchResult> {
  return invoke('search_cards', { args: { query } });
}

export async function getCard(id: string): Promise<CardResponse> {
  return invoke('get_card', { args: { id } });
}

export async function getCardsBySet(setCode: string): Promise<CardResponse[]> {
  return invoke('get_cards_by_set', { args: { set_code: setCode } });
}

export async function getAllSets(): Promise<SetResponse[]> {
  return invoke('get_all_sets');
}

export async function getCardPrices(cardId: string): Promise<PriceResponse> {
  return invoke('get_card_prices', { args: { card_id: cardId } });
}

// ============================================================================
// COLLECTION OPERATIONS
// ============================================================================

export async function getCollection(): Promise<CollectionItem[]> {
  return invoke('get_collection');
}

export async function addToCollection(args: AddToCollectionArgs): Promise<CollectionItemResponse> {
  return invoke('add_to_collection', { args });
}

export async function removeFromCollection(cardId: string): Promise<{ success: boolean }> {
  return invoke('remove_from_collection', { args: { card_id: cardId } });
}

export async function updateCollectionItem(
  cardId: string,
  quantity: number,
  condition?: string,
  notes?: string
): Promise<CollectionItem> {
  return invoke('update_collection_item', { args: { card_id: cardId, quantity, condition, notes } });
}

// ============================================================================
// DECK OPERATIONS
// ============================================================================

export async function listDecks(): Promise<DeckResponse[]> {
  return invoke('list_decks');
}

export async function getDeck(id: number): Promise<DeckResponse> {
  return invoke('get_deck', { args: { id: String(id) } });
}

export async function createDeck(args: CreateDeckArgs): Promise<CreatedDeckResponse> {
  return invoke('create_deck', { args });
}

export async function deleteDeck(id: number): Promise<{ success: boolean }> {
  return invoke('delete_deck', { args: { id: String(id) } });
}

export async function updateDeck(
  id: number,
  name?: string,
  format?: string,
  description?: string
): Promise<DeckResponse> {
  return invoke('update_deck', { args: { id: String(id), name, format, description } });
}

export async function addCardToDeck(
  deckId: number,
  cardId: string,
  quantity: number = 1,
  position?: number
): Promise<DeckCardResponse> {
  return invoke('add_card_to_deck', { args: { deck_id: String(deckId), card_id: cardId, quantity, position } });
}

export async function removeCardFromDeck(deckId: number, cardId: string): Promise<{ success: boolean }> {
  return invoke('remove_card_from_deck', { args: { deck_id: String(deckId), card_id: cardId } });
}

export async function updateDeckCardQuantity(
  deckId: number,
  cardId: string,
  quantity: number
): Promise<DeckCardResponse> {
  return invoke('update_deck_card_quantity', { args: { deck_id: String(deckId), card_id: cardId, quantity } });
}

export async function reorderDeckCards(deckId: number, cardIds: string[]): Promise<DeckCardResponse[]> {
  return invoke('reorder_deck_cards', { args: { deck_id: String(deckId), card_ids: cardIds } });
}

// ============================================================================
// DECK ANALYSIS
// ============================================================================

export async function getDeckManaCurve(deckId: number): Promise<DeckManaCurve> {
  return invoke('get_deck_mana_curve', { args: { id: String(deckId) } });
}

export async function goldfishDeck(deckId: number, turns: number = 10): Promise<DeckGoldfishResult> {
  return invoke('goldfish_deck', { args: { deck_id: String(deckId), turns } });
}

export async function checkDeckLegality(deckId: number, format: string): Promise<DeckLegalityResult> {
  return invoke('check_deck_legality', { args: { deck_id: String(deckId), format } });
}

// ============================================================================
// LORE OPERATIONS
// ============================================================================

export async function loadLoreEntries(args?: LoadLoreArgs): Promise<LoreEntry[]> {
  return invoke('load_lore_entries', { args: args || {} });
}

export async function searchLore(args: { query: string }): Promise<LoreEntry[]> {
  return invoke('search_lore', { args });
}

export async function getLoreEntry(id: number): Promise<LoreEntryResponse> {
  return invoke('get_lore_entry', { args: { id: String(id) } });
}

export async function createLoreEntry(
  title: string,
  loreType: string,
  content: string,
  relatedCards?: string[],
  metadata?: any
): Promise<LoreEntryResponse> {
  return invoke('create_lore_entry', { args: { title, lore_type: loreType, content, related_cards: relatedCards || [], metadata: metadata || {} } });
}

// ============================================================================
// IMPORT / BULK OPER OPERATIONS
// ============================================================================

export async function importAllCards(): Promise<BulkImportResult> {
  return invoke('import_all_cards');
}

export async function importSets(): Promise<BulkImportResult> {
  return invoke('import_sets');
}

export async function importPrices(): Promise<BulkImportResult> {
  return invoke('import_prices');
}

export async function getImportStatus(): Promise<ImportStatus> {
  return invoke('get_import_status');
}

export async function cancelImport(): Promise<{ success: boolean }> {
  return invoke('cancel_import');
}

// ============================================================================
// UTILITY: Invoke wrapper with error handling
// ============================================================================

export class TauriAPIError extends Error {
  constructor(
    message: string,
    public readonly code?: string,
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'TauriAPIError';
  }
}

export async function safeInvoke<T>(command: string, args?: any): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new TauriAPIError(`Tauri command "${command}" failed: ${message}`, 'TAURI_INVOKE_ERROR', error);
  }
}

// Re-export invoke for direct use if needed
export { invoke } from '@tauri-apps/api/core';