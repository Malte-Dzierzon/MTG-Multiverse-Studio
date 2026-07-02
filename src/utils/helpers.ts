// MTG Multiverse Studio — Utility Functions

import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Combines class names with tailwind-merge for proper Tailwind class precedence
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Format mana cost string for display (e.g. "{2}{W}{W}" → "2WW")
 */
export function formatManaCost(manaCost?: string): string {
  if (!manaCost) return '';
  return manaCost
    .replace(/{/g, '')
    .replace(/}/g, '')
    .replace(/\//g, '/');
}

/**
 * Parse mana cost into individual symbols
 */
export function parseManaSymbols(manaCost?: string): string[] {
  if (!manaCost) return [];
  const matches = manaCost.match(/{([^}]+)}/g);
  return matches ? matches.map(m => m.slice(1, -1)) : [];
}

/**
 * Format price for display
 */
export function formatPrice(price?: string, currency: 'usd' | 'eur' = 'eur'): string {
  if (!price || price === 'null') return '—';
  const num = parseFloat(price);
  if (isNaN(num)) return '—';
  return new Intl.NumberFormat('de-DE', {
    style: 'currency',
    currency: currency.toUpperCase(),
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(num);
}

/**
 * Format number with German locale
 */
export function formatNumber(num: number): string {
  return new Intl.NumberFormat('de-DE').format(num);
}

/**
 * Format date for display
 */
export function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('de-DE', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  }).format(date);
}

/**
 * Format relative time (e.g. "vor 2 Stunden")
 */
export function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'gerade eben';
  if (diffMins < 60) return `vor ${diffMins} Min.`;
  if (diffHours < 24) return `vor ${diffHours} Std.`;
  if (diffDays < 7) return `vor ${diffDays} Tagen`;
  return formatDate(dateString);
}

/**
 * Get rarity color class
 */
export function getRarityColor(rarity: string): string {
  switch (rarity.toLowerCase()) {
    case 'mythic': return 'text-amber-400';
    case 'rare': return 'text-gold-300';
    case 'uncommon': return 'text-silver-300';
    case 'common': return 'text-stone-400';
    case 'special': return 'text-purple-400';
    case 'bonus': return 'text-pink-400';
    default: return 'text-muted';
  }
}

/**
 * Get rarity background class for badges
 */
export function getRarityBg(rarity: string): string {
  switch (rarity.toLowerCase()) {
    case 'mythic': return 'bg-amber-900/50 border-amber-400';
    case 'rare': return 'bg-yellow-900/50 border-yellow-300';
    case 'uncommon': return 'bg-gray-700/50 border-gray-300';
    case 'common': return 'bg-stone-700/50 border-stone-400';
    case 'special': return 'bg-purple-900/50 border-purple-400';
    case 'bonus': return 'bg-pink-900/50 border-pink-400';
    default: return 'bg-interactive border-wood';
  }
}

/**
 * Debounce function
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Throttle function
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Generate unique ID
 */
export function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Clamp number between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Sleep utility
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Deep clone
 */
export function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj));
}

/**
 * Group array by key
 */
export function groupBy<T>(array: T[], key: keyof T | ((item: T) => string)): Record<string, T[]> {
  return array.reduce((groups, item) => {
    const groupKey = typeof key === 'function' ? key(item) : String(item[key]);
    groups[groupKey] = groups[groupKey] || [];
    groups[groupKey].push(item);
    return groups;
  }, {} as Record<string, T[]>);
}

/**
 * Sort cards by CMC, then name
 */
export function sortCardsByCmc<T extends { cmc: number; name: string }>(cards: T[]): T[] {
  return [...cards].sort((a, b) => {
    if (a.cmc !== b.cmc) return a.cmc - b.cmc;
    return a.name.localeCompare(b.name);
  });
}

/**
 * Sort cards by color identity (WUBRG order)
 */
export function sortCardsByColor<T extends { colors: string[]; color_identity: string[] }>(cards: T[]): T[] {
  const colorOrder = { W: 0, U: 1, B: 2, R: 3, G: 4, C: 5 };
  return [...cards].sort((a, b) => {
    const aColors = a.color_identity.length > 0 ? a.color_identity : a.colors;
    const bColors = b.color_identity.length > 0 ? b.color_identity : b.colors;
    
    const aPrimary = aColors[0] || 'C';
    const bPrimary = bColors[0] || 'C';
    
    return (colorOrder[aPrimary as keyof typeof colorOrder] || 5) - 
           (colorOrder[bPrimary as keyof typeof colorOrder] || 5);
  });
}

/**
 * Calculate deck stats
 */
export function calculateDeckStats(cards: { card: { cmc: number; colors: string[]; type_line: string }; quantity: number }[]) {
  let totalCards = 0;
  let totalCmc = 0;
  const colorCount = { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 };
  const typeCount = { creature: 0, instant: 0, sorcery: 0, enchantment: 0, artifact: 0, planeswalker: 0, land: 0, other: 0 };
  const cmcCurve: Record<number, number> = {};

  cards.forEach(({ card, quantity }) => {
    totalCards += quantity;
    totalCmc += card.cmc * quantity;

    // CMC curve
    const cmc = Math.floor(card.cmc);
    cmcCurve[cmc] = (cmcCurve[cmc] || 0) + quantity;

    // Color identity
    const colors = card.colors.length > 0 ? card.colors : ['C'];
    colors.forEach(c => {
      const key = c.toUpperCase() as keyof typeof colorCount;
      if (key in colorCount) colorCount[key] += quantity;
    });

    // Type count
    const type = card.type_line.toLowerCase();
    if (type.includes('creature')) typeCount.creature += quantity;
    else if (type.includes('instant')) typeCount.instant += quantity;
    else if (type.includes('sorcery')) typeCount.sorcery += quantity;
    else if (type.includes('enchantment')) typeCount.enchantment += quantity;
    else if (type.includes('artifact')) typeCount.artifact += quantity;
    else if (type.includes('planeswalker')) typeCount.planeswalker += quantity;
    else if (type.includes('land')) typeCount.land += quantity;
    else typeCount.other += quantity;
  });

  return {
    totalCards,
    avgCmc: totalCards > 0 ? totalCmc / totalCards : 0,
    colorCount,
    typeCount,
    cmcCurve,
  };
}