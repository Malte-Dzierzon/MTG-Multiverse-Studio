import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { GlassPanel } from '../components/ui/GlassPanel';
import { Button } from '../components/ui/Button';
import { CardPreview } from '../components/ui/CardPreview';
import { SearchInput } from '../components/ui/SearchInput';
import { ChevronLeft, ChevronRight, Trash2, Save, Download, Copy, Edit, Loader2, X, Sparkles, Archive, FlaskConical, BookOpen, Settings } from 'lucide-react';
import { getDeck, getCollection, updateDeck, deleteDeck, goldfishDeck } from '../services/api';
import { cn, formatNumber } from '../utils/helpers';
import { motion, AnimatePresence } from 'framer-motion';

export default function DeckDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [selectedCard, setSelectedCard] = useState<any>(null);
  const [showGoldfish, setShowGoldfish] = useState(false);
  const [goldfishResult, setGoldfishResult] = useState<any>(null);

  const { data: deck, isLoading: deckLoading, refetch: refetchDeck } = useQuery({
    queryKey: ['deck', id],
    queryFn: () => getDeck(id!),
    enabled: !!id,
  });

  const { data: collection } = useQuery({
    queryKey: ['collection'],
    queryFn: getCollection,
  });

  const handleGoldfish = async () => {
    setShowGoldfish(true);
    try {
      const result = await goldfishDeck(id!);
      setGoldfishResult(result);
    } catch (error) {
      console.error('Goldfish failed:', error);
    }
  };

  const handleDeleteDeck = async () => {
    if (window.confirm('Deck wirklich löschen?')) {
      try {
        await deleteDeck(id!);
        navigate('/deckbuilder');
      } catch (error) {
        console.error('Failed to delete deck:', error);
      }
    }
  };

  if (deckLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-[var(--color-parchment)]" />
      </div>
    );
  }

  if (!deck) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <GlassPanel variant="strong" className="p-8 text-center">
          <p className="text-[var(--color-text-muted)]">Deck nicht gefunden</p>
          <Button onClick={() => navigate('/deckbuilder')} className="mt-4">
            Zurück zum Deck-Builder
          </Button>
        </GlassPanel>
      </div>
    );
  }

  const totalCards = deck.cards?.reduce((sum: number, c: any) => sum + c.quantity, 0) || 0;
  const avgCmc = deck.cards?.reduce((sum: number, c: any) => sum + (c.card.cmc || 0) * c.quantity, 0) / totalCards || 0;

  return (
    <div className="min-h-screen pb-24">
      {/* Header */}
      <header className="sticky top-0 z-40 bg-[var(--color-bg-base)]/80 backdrop-blur-xl border-b border-[var(--color-wood)]/10">
        <div className="max-w-7xl mx-auto px-4 md:px-8 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Button variant="ghost" size="sm" icon={<ChevronLeft className="h-4 w-4" />} onClick={() => navigate('/deckbuilder')} />
              <div>
                <h1 className="heading-1 font-serif text-2xl md:text-3xl">{deck.name}</h1>
                <p className="text-[var(--color-text-muted)] text-sm font-mono">
                  {deck.format || 'Kein Format'} • {totalCards} Karten • Ø CMC: {avgCmc.toFixed(2)}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button variant="secondary" size="sm" icon={<Sparkles className="h-4 w-4" />} onClick={handleGoldfish} loading={showGoldfish}>
                Goldfisch
              </Button>
              <Button variant="secondary" size="sm" icon={<Download className="h-4 w-4" }}>
                Exportieren
              </Button>
              <Button variant="danger" size="sm" icon={<Trash2 className="h-4 w-4" />} onClick={handleDeleteDeck}>
                Löschen
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Content */}
      <main className="max-w-7xl mx-auto px-4 md:px-8 py-6">
        {/* Deck Stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
          <GlassPanel variant="strong" className="p-4">
            <div className="flex items-center gap-3">
              <Archive className="h-8 w-8 text-[var(--color-parchment)]" />
              <div>
                <p className="text-sm text-[var(--color-text-muted)]">Hauptdeck</p>
                <p className="text-2xl font-bold text-[var(--color-parchment)]">{totalCards}</p>
              </div>
            </div>
          </GlassPanel>
          <GlassPanel variant="strong" className="p-4">
            <div className="flex items-center gap-3">
              <FlaskConical className="h-8 w-8 text-[var(--color-parchment)]" />
              <div>
                <p className="text-sm text-[var(--color-text-muted)]">Ø Mana-Wert</p>
                <p className="text-2xl font-bold text-[var(--color-parchment)]">{avgCmc.toFixed(2)}</p>
              </div>
            </div>
          </GlassPanel>
          <GlassPanel variant="strong" className="p-4">
            <div className="flex items-center gap-3">
              <Sparkles className="h-8 w-8 text-[var(--color-parchment)]" />
              <div>
                <p className="text-sm text-[var(--color-text-muted)]">Seitenboard</p>
                <p className="text-2xl font-bold text-[var(--color-parchment)]">0</p>
              </div>
            </div>
          </GlassPanel>
          <GlassPanel variant="strong" className="p-4">
            <div className="flex items-center gap-3">
              <BookOpen className="h-8 w-8 text-[var(--color-parchment)]" />
              <div>
                <p className="text-sm text-[var(--color-text-muted)]">Preis (ca.)</p>
                <p className="text-2xl font-bold text-[var(--color-parchment)]">
                  ${deck.cards?.reduce((sum: number, c: any) => sum + (parseFloat(c.card.prices?.usd || '0') * c.quantity), 0).toFixed(2) || '0.00'}
                </p>
              </div>
            </div>
          </GlassPanel>
        </div>

        {/* Mana Curve */}
        <GlassPanel variant="strong" className="p-5 mb-6">
          <h2 className="font-serif text-lg font-semibold mb-4">Mana-Kurve</h2>
          <div className="flex items-end gap-1 h-32">
            {Array.from({ length: 8 }, (_, i) => i).map((cmc) => {
              const count = deck.cards?.reduce((sum: number, c: any) => sum + ((c.card.cmc === cmc || (cmc === 7 && c.card.cmc >= 7)) ? c.quantity : 0), 0) || 0;
              const height = totalCards > 0 ? (count / totalCards) * 100 : 0;
              return (
                <div key={cmc} className="flex-1 flex flex-col items-center">
                  <motion.div
                    initial={{ height: 0 }}
                    animate={{ height: `${Math.max(height, 5)}%` }}
                    className="w-full bg-[var(--color-parchment)] rounded-t-sm transition-all duration-500"
                    style={{ minHeight: '4px' }}
                  />
                  <span className="font-mono text-xs text-[var(--color-text-muted)] mt-1">{cmc === 7 ? '7+' : cmc}</span>
                  <span className="font-mono text-xs text-[var(--color-parchment)]">{count}</span>
                </div>
              );
            })}
          </div>
        </GlassPanel>

        {/* Card List */}
        <GlassPanel variant="strong" className="p-5">
          <h2 className="font-serif text-lg font-semibold mb-4">Karten ({deck.cards?.length || 0})</h2>
          <div className="space-y-2">
            {deck.cards?.map((entry: any, index: number) => (
              <motion.div
                key={entry.card.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.2, delay: index * 0.02 }}
              >
                <GlassPanel variant="card" className="p-3 flex items-center gap-4" onClick={() => setSelectedCard(entry.card)}>
                  <div className="flex-shrink-0 text-[var(--color-text-muted)] font-mono text-sm w-8">
                    #{index + 1}
                  </div>
                  <CardPreview card={entry.card} size="small" variant="deck-item" />
                  <div className="flex-1 min-w-0">
                    <p className="font-medium truncate">{entry.card.name}</p>
                    <p className="text-xs text-[var(--color-text-muted)] font-mono">
                      {entry.card.set_name} • {entry.card.type_line?.split(' ')[0]} • {entry.card.mana_cost?.replace(/{/g, '').replace(/}/g, '') || '—'}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={cn('px-2 py-0.5 rounded font-mono text-xs border', 'bg-[var(--color-parchment)] text-[var(--color-bg-base)]')}>
                      x{entry.quantity}
                    </span>
                    {collection?.some((c: any) => c.card.id === entry.card.id && c.quantity >= entry.quantity) ? (
                      <span className="px-2 py-0.5 rounded font-mono text-xs text-green-400 bg-green-400/10 border border-green-400/20">
                        In Sammlung
                      </span>
                    ) : (
                      <span className="px-2 py-0.5 rounded font-mono text-xs text-amber-400 bg-amber-400/10 border border-amber-400/20">
                        Fehlt
                      </span>
                    )}
                  </div>
                </GlassPanel>
              </motion.div>
            ))}
          </div>
        </GlassPanel>
      </main>

      {/* Card Detail Modal */}
      <AnimatePresence>
        {selectedCard && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
            onClick={() => setSelectedCard(null)}
          >
            <motion.div
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative max-w-md w-full max-h-[90vh] overflow-y-auto"
              onClick={(e) => e.stopPropagation()}
            >
              <CardPreview card={selectedCard} size="large" showPrice={true} variant="levitate" />
              <div className="absolute top-4 right-4">
                <Button variant="glass" size="sm" icon={<X className="h-4 w-4" />} onClick={() => setSelectedCard(null)} />
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Goldfish Modal */}
      <AnimatePresence>
        {showGoldfish && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
            onClick={() => setShowGoldfish(false)}
          >
            <motion.div
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative max-w-4xl w-full max-h-[90vh] overflow-y-auto bg-[var(--color-bg-surface)] rounded-lg"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="p-6">
                <div className="flex items-center justify-between mb-6">
                  <h2 className="font-serif text-2xl font-semibold">Goldfisch-Test</h2>
                  <Button variant="glass" size="sm" icon={<X className="h-4 w-4" />} onClick={() => setShowGoldfish(false)} />
                </div>

                {goldfishResult && (
                  <div className="space-y-6">
                    <div>
                      <h3 className="font-medium mb-3">Start-Hand</h3>
                      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                        {goldfishResult.hand.map((card: any) => (
                          <CardPreview key={card.id} card={card} size="small" variant="default" />
                        ))}
                      </div>
                    </div>

                    <div>
                      <h3 className="font-medium mb-3">Bibliothek: {goldfishResult.library} Karten</h3>
                    </div>

                    <div>
                      <h3 className="font-medium mb-3">Züge</h3>
                      <div className="space-y-4">
                        {goldfishResult.turns.map((turn: any) => (
                          <GlassPanel key={turn.turn} variant="card" className="p-4">
                            <div className="flex items-center justify-between mb-3">
                              <h4 className="font-semibold">Zug {turn.turn}</h4>
                              <div className="flex items-center gap-2">
                                {Object.entries(turn.mana_available).map(([color, amount]) => (
                                  <span key={color} className="px-1.5 py-0.5 rounded text-xs font-mono bg-[var(--color-parchment)] text-[var(--color-bg-base)]">
                                    {color}: {amount}
                                  </span>
                                ))}
                              </div>
                            </div>
                            <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                              {turn.drawn.map((card: any) => (
                                <CardPreview key={card.id} card={card} size="small" variant="default" />
                              ))}
                            </div>
                            {turn.played.length > 0 && (
                              <div className="mt-3">
                                <p className="text-sm text-[var(--color-text-muted)] mb-2">Gesetzt:</p>
                                <div className="flex flex-wrap gap-2">
                                  {turn.played.map((card: any) => (
                                    <span key={card.id} className="px-2 py-1 text-xs bg-[var(--color-bg-interactive)] rounded border border-[var(--color-wood)]">
                                      {card.name}
                                    </span>
                                  ))}
                                </div>
                              </div>
                            )}
                          </GlassPanel>
                        ))}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}