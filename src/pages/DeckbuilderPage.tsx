import { useState } from 'react';
import { CardPreview } from '../components/ui/CardPreview';
import { SearchInput } from '../components/ui/SearchInput';
import { Loader2, X } from 'lucide-react';
import { searchCards } from '../services/api';
import { useQuery } from '@tanstack/react-query';

export default function DeckbuilderPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCard, setSelectedCard] = useState<any>(null);

  // Search cards
  const { data: searchResults, isLoading: searchLoading } = useQuery({
    queryKey: ['search', searchQuery],
    queryFn: () => searchCards(searchQuery),
    enabled: searchQuery.length >= 2,
    staleTime: 1000 * 60 * 5,
  });

  const filteredResults = searchResults?.cards
    ?.filter((card: any) => card.name.toLowerCase().includes(searchQuery.toLowerCase()))
    .slice(0, 10) || [];

  return (
    <div className="min-h-screen pb-24">
      <div className="max-w-7xl mx-auto px-4 md:px-8 py-6">
        <h1 className="heading-1 font-serif">Deck-Builder (Demo)</h1>
        <p className="text-[var(--color-text-muted)] mt-2">
          This is a simplified version. In the full implementation you'll be able to:
        </p>
        <ul className="list-disc pl-6 mt-4 text-[var(--color-text-muted)] space-y-1">
          <li>Search and add cards to your deck</li>
          <li>Manage card quantities (up to 4 per card)</li>
          <li>Remove cards from your deck</li>
          <li>Analyze your deck's mana curve and composition</li>
          <li>Save and load decklists</li>
        </ul>

        <div className="mt-8">
          <h2 className="heading-2 font-serif text-xl mb-4">Test Card Search</h2>
          <SearchInput
            value={searchQuery}
            onChange={setSearchQuery}
            placeholder="Tome of Secrets..."
            autoFocus
          />

          {searchLoading ? (
            <div className="mt-4 flex items-center justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-[var(--color-parchment)]" />
            </div>
          ) : filteredResults.length > 0 ? (
            <div className="mt-4 grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
              {filteredResults.map((card: any) => (
                <CardPreview
                  key={card.id}
                  card={card}
                  size="small"
                  variant="levitate"
                  onClick={() => setSelectedCard(card)}
                />
              ))}
            </div>
          ) : searchQuery.length >= 2 ? (
            <div className="mt-4 text-center py-8 text-[var(--color-text-muted)]">
              No cards found
            </div>
          ) : (
            <div className="mt-4 text-center py-8 text-[var(--color-text-muted)]">
              Search for cards to add to your deck
            </div>
          )}
        </div>

        {selectedCard && (
          <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
            <div className="relative max-w-2xl w-full bg-[var(--color-bg-surface)] rounded-lg p-6">
              <button
                onClick={() => setSelectedCard(null)}
                className="absolute top-4 right-4 text-[var(--color-text-muted)] hover:text-[var(--color-parchment)]"
              >
                <X className="h-6 w-6" />
              </button>
              <h2 className="font-serif text-2xl font-semibold mb-4 pr-8">
                {selectedCard.name}
              </h2>
              <div className="flex gap-6">
                <CardPreview card={selectedCard} size="large" showPrice={false} variant="levitate" />
                <div className="flex-1">
                  <div className="space-y-3">
                    <p>
                      <span className="text-[var(--color-text-muted)]">Mana Cost: </span>
                      {selectedCard.mana_cost}
                    </p>
                    <p>
                      <span className="text-[var(--color-text-muted)]">Type: </span>
                      {selectedCard.type_line}
                    </p>
                    <p>
                      <span className="text-[var(--color-text-muted)]">Rarity: </span>
                      {selectedCard.rarity}
                    </p>
                    <p>
                      <span className="text-[var(--color-text-muted)]">Set: </span>
                      {selectedCard.set_name}
                    </p>
                    {selectedCard.cmc > 0 && (
                      <p>
                        <span className="text-[var(--color-text-muted)]">Converted Mana Cost: </span>
                        {selectedCard.cmc}
                      </p>
                    )}
                  </div>
                  {selectedCard.card_text && (
                    <div className="mt-4">
                      <h3 className="font-medium mb-2">Card Text</h3>
                      <p className="text-sm text-[var(--color-text-muted)] whitespace-pre-wrap">
                        {selectedCard.card_text}
                      </p>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}