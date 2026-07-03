// Lore Reference Utilities
// Parses [[CardName]] and @CardName references and converts them to clickable links

import { MANA_COLORS } from '../types';

export interface CardReference {
  type: 'bracket' | 'at';
  name: string;
  start: number;
  end: number;
}

/**
 * Extracts card references from lore text
 * Supports [[CardName]] and @CardName patterns
 */
export function extractCardReferences(text: string): CardReference[] {
  const references: CardReference[] = [];
  
  // Pattern: [[CardName]]
  const bracketRegex = /\[\[([^\]]+)\]\]/g;
  let match: RegExpExecArray | null;
  while ((match = bracketRegex.exec(text)) !== null) {
    references.push({
      type: 'bracket',
      name: match[1].trim(),
      start: match.index,
      end: match.index + match[0].length,
    });
  }
  
  // Pattern: @CardName (word starting with @ followed by uppercase letter)
  const atRegex = /@([A-Z][a-zA-Z]*(?:\s+[A-Z][a-zA-Z]*)*)/g;
  while ((match = atRegex.exec(text)) !== null) {
    references.push({
      type: 'at',
      name: match[1].trim(),
      start: match.index,
      end: match.index + match[0].length,
    });
  }
  
  return references.sort((a, b) => a.start - b.start);
}

/**
 * Replaces card references in text with clickable link components
 * Returns an array of React nodes (text and links interleaved)
 */
export function parseLoreContentWithLinks(
  text: string,
  onCardClick: (cardName: string) => void
): React.ReactNode[] {
  const references = extractCardReferences(text);
  if (references.length === 0) {
    return [text];
  }
  
  const nodes: React.ReactNode[] = [];
  let lastEnd = 0;
  
  for (const ref of references) {
    // Add text before the reference
    if (ref.start > lastEnd) {
      nodes.push(text.slice(lastEnd, ref.start));
    }
    
    // Add clickable card reference
    nodes.push(
      <span
        key={`${ref.type}-${ref.name}-${ref.start}`}
        className="text-[var(--color-parchment)] underline cursor-pointer hover:text-[var(--color-gold)] transition-colors"
        onClick={() => onCardClick(ref.name)}
        title={`Zur Karte: ${ref.name}`}
      >
        {ref.type === 'bracket' ? `[${ref.name}]` : `@${ref.name}`}
      </span>
    );
    
    lastEnd = ref.end;
  }
  
  // Add remaining text
  if (lastEnd < text.length) {
    nodes.push(text.slice(lastEnd));
  }
  
  return nodes;
}

/**
 * Simple version that returns a CardPreview component for a given card name
 * navigates to collection page with search
 */
export function navigateToCard(cardName: string) {
  // Navigate to collection page with search query
  window.location.href = `/collection?search=${encodeURIComponent(cardName)}`;
}

/**
 * Get mana color styling for a card reference
 */
export function getManaColorStyle(symbol: string) {
  const color = MANA_COLORS[symbol as keyof typeof MANA_COLORS];
  return color || { bg: '#8c837e', text: '#161515', name: symbol };
}