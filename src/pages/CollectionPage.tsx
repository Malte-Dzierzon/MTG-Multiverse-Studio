import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { GlassPanel } from '../components/ui/GlassPanel';
import { Button } from '../components/ui/Button';
import { CardPreview } from '../components/ui/CardPreview';
import { SearchInput } from '../components/ui/SearchInput';
import { Plus, Trash2, Loader2, X, Filter, Download, ChevronDown, ChevronUp, Archive, Edit } from 'lucide-react';
import { searchCards, getCollection, addToCollection, removeFromCollection, importAllCards, getImportStatus } from '../services/api';
import { formatNumber } from '../utils/helpers';
import { motion, AnimatePresence } from 'framer-motion';

export default function CollectionPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCard, setSelectedCard] = useState<any>(null);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [sortBy, setSortBy] = useState<'name' | 'cmc' | 'color' | 'rarity' | 'price'>('name');
  const [filterColor, setFilterColor] = useState<string>('all');
  const [filterRarity, setFilterRarity] = useState<string>('all');
  const [showFilters, setShowFilters] = useState(false);
  const [importRunning, setImportRunning] = useState(false);
  const [importProgress, setImportProgress] = useState<{progress: number; current_set?: string} | null>(null);

  // Search cards
  const { data: searchResults, isLoading: searchLoading } = useQuery({
    queryKey: ['search', searchQuery],
    queryFn: () => searchCards(searchQuery),
    enabled: searchQuery.length >= 2,
    staleTime: 1000 * 60 * 5,
  });

  // Load collection
  const { data: collection, isLoading: collectionLoading, refetch: refetchCollection } = useQuery({
    queryKey: ['collection'],
    queryFn: getCollection,
  });

  // Poll import status
  useEffect(() => {
    if (!importRunning) return;
    const interval = setInterval(async () => {
      const status = await getImportStatus();
      setImportProgress({ progress: status.progress, current_set: status.current_set });
      if (!status.running) {
        setImportRunning(false);
        setImportProgress(null);
        refetchCollection();
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [importRunning, refetchCollection]);

  const handleImportAll = async () => {
    setImportRunning(true);
    setImportProgress({ progress: 0 });
    try {
      await importAllCards();
    } catch (error) {
      console.error('Import failed:', error);
      setImportRunning(false);
      setImportProgress(null);
    }
  };

  const handleAddToCollection = async (card: any) => {
    try {
      await addToCollection({ card_id: card.id, quantity: 1, condition: 'nm' });
      refetchCollection();
    } catch (error) {
      console.error('Failed to add to collection:', error);
    }
  };

  const handleRemoveFromCollection = async (cardId: string) => {
    try {
      await removeFromCollection(cardId);
      refetchCollection();
    } catch (error) {
      console.error('Failed to remove from collection:', error);
    }
  };

  const getCollectionQuantity = (cardId: string) => {
    return collection?.find((c: any) => c.card.id === cardId)?.quantity || 0;
  };

  const filteredResults = searchResults?.cards
    ?.filter((card: any) => {
      if (filterColor !== 'all' && !card.colors?.includes(filterColor)) return false;
      if (filterRarity !== 'all' && card.rarity?.toLowerCase() !== filterRarity) return false;
      return true;
    })
    ?.sort((a: any, b: any) => {
      switch (sortBy) {
        case 'cmc': return a.cmc - b.cmc || a.name.localeCompare(b.name);
        case 'color': return (a.colors?.[0] || '').localeCompare(b.colors?.[0] || '');
        case 'rarity': return (rarityRank(b.rarity) - rarityRank(a.rarity));
        case 'price': return (parseFloat(b.prices?.usd || '0') - parseFloat(a.prices?.usd || '0'));
        default: return a.name.localeCompare(b.name);
      }
    }) || [];

  const rarityRank = (rarity: string) => {
    const order: Record<string, number> = { mythic: 4, rare: 3, uncommon: 2, common: 1, special: 0, bonus: 0 };
    return order[rarity?.toLowerCase()] || 0;
  };

  return (
    <div className="min-h-screen pb-24">
      {/* Header */}
      <header className="sticky top-0 z-40 bg-[var(--color-bg-base)]/80 backdrop-blur-xl border-b border-[var(--color-wood)]/10">
        <div className="max-w-7xl mx-auto px-4 md:px-8 py-4">
          <div className="flex flex-col md:flex-row md:items-center gap-4">
            <div className="flex items-center gap-3">
              <Archive className="h-8 w-8 text-[var(--color-parchment)]" />
              <div>
                <h1 className="heading-1 font-serif text-2xl md:text-3xl">Sammlung</h1>
                <p className="text-[var(--color-text-muted)] text-sm font-mono">
                  {collection?.length || 0} Karten • {formatNumber(collection?.reduce((sum: number, c: any) => sum + c.quantity, 0) || 0)} gesamt
                </p>
              </div>
            </div>

            <div className="flex-1 max-w-xl">
              <SearchInput
                value={searchQuery}
                onChange={setSearchQuery}
                placeholder="Karten suchen (Name, Mana, Typ)..."
                autoFocus
              />
            </div>

            <div className="flex items-center gap-2 md:ml-auto">
              <Button variant="glass" size="sm" icon={<Filter className="h-4 w-4" />} onClick={() => setShowFilters(!showFilters)}>
                Filter
              </Button>
              <Button variant="secondary" size="sm" icon={<Download className="h-4 w-4" />} onClick={handleImportAll} loading={importRunning}>
                Vollimport
              </Button>
            </div>
          </div>

          {/* Filters */}
          <AnimatePresence>
            {showFilters && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                className="mt-4 overflow-hidden"
              >
                <GlassPanel variant="strong" className="p-4">
                  <div className="flex flex-wrap items-center gap-4">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-xs text-[var(--color-text-muted)]">Farbe:</span>
                      <select
                        value={filterColor}
                        onChange={(e) => setFilterColor(e.target.value)}
                        className="glass-panel px-3 py-1.5 text-sm font-mono text-[var(--color-text-main)] bg-transparent border-none outline-none focus:ring-1 focus:ring-[var(--color-parchment)]"
                      >
                        <option value="all">Alle</option>
                        <option value="W">Weiß</option>
                        <option value="U">Blau</option>
                        <option value="B">Schwarz</option>
                        <option value="R">Rot</option>
                        <option value="G">Grün</option>
                        <option value="C">Farblos</option>
                      </select>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-xs text-[var(--color-text-muted)]">Seltenheit:</span>
                      <select
                        value={filterRarity}
                        onChange={(e) => setFilterRarity(e.target.value)}
                        className="glass-panel px-3 py-1.5 text-sm font-mono text-[var(--color-text-main)] bg-transparent border-none outline-none focus:ring-1 focus:ring-[var(--color-parchment)]"
                      >
                        <option value="all">Alle</option>
                        <option value="mythic">Mythic</option>
                        <option value="rare">Rare</option>
                        <option value="uncommon">Uncommon</option>
                        <option value="common">Common</option>
                      </select>
                    </div>
                    <div className="flex items-center gap-2 ml-auto">
                      <span className="font-mono text-xs text-[var(--color-text-muted)]">Sortieren:</span>
                      <select
                        value={sortBy}
                        onChange={(e) => setSortBy(e.target.value as any)}
                        className="glass-panel px-3 py-1.5 text-sm font-mono text-[var(--color-text-main)] bg-transparent border-none outline-none focus:ring-1 focus:ring-[var(--color-parchment)]"
                      >
                        <option value="name">Name</option>
                        <option value="cmc">Mana-Wert</option>
                        <option value="color">Farbe</option>
                        <option value="rarity">Seltenheit</option>
                        <option value="price">Preis</option>
                      </select>
                    </div>
                  </div>
                </GlassPanel>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Import Progress */}
          {importRunning && importProgress && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mt-4 glass-panel-strong p-4"
            >
              <div className="flex items-center gap-3">
                <Loader2 className="h-5 w-5 animate-spin text-[var(--color-parchment)]" />
                <div className="flex-1">
                  <div className="flex justify-between text-sm mb-1">
                    <span className="font-mono">Import läuft...</span>
                    <span className="font-mono text-[var(--color-parchment)]">{Math.round(importProgress.progress)}%</span>
                  </div>
                  <div className="h-2 bg-[var(--color-bg-surface)] rounded-full overflow-hidden">
                    <motion.div
                      className="h-full bg-[var(--color-parchment)]"
                      initial={{ width: 0 }}
                      animate={{ width: `${importProgress.progress}%` }}
                      transition={{ duration: 0.3 }}
                    />
                  </div>
                  {importProgress.current_set && (
                    <p className="text-xs text-[var(--color-text-muted)] mt-1 font-mono">
                      Set: {importProgress.current_set}
                    </p>
                  )}
                </div>
              </div>
            </motion.div>
          )}
        </div>
      </header>

      {/* Content */}
      <main className="max-w-7xl mx-auto px-4 md:px-8 py-6">
        {searchQuery.length >= 2 ? (
          // Search Results
          <div>
            <div className="flex items-center justify-between mb-4">
              <h2 className="heading-2 font-serif text-xl">
                Suchergebnisse ({searchResults?.total || 0})
              </h2>
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  icon={viewMode === 'grid' ? <ChevronDown className="h-4 w-4" /> : <ChevronUp className="h-4 w-4" />}
                  onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}
                />
              </div>
            </div>

            {searchLoading ? (
              <GlassPanel variant="strong" className="h-64 flex items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-[var(--color-parchment)]" />
              </GlassPanel>
            ) : filteredResults.length === 0 ? (
              <GlassPanel variant="strong" className="h-64 flex items-center justify-center text-[var(--color-text-muted)]">
                Keine Karten gefunden
              </GlassPanel>
            ) : (
              <AnimatePresence mode="popLayout">
                {viewMode === 'grid' ? (
                  <motion.div
                    key="grid"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4"
                  >
                    {filteredResults.map((card: any) => (
                      <motion.div
                        key={card.id}
                        initial={{ opacity: 0, scale: 0.9, y: 20 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        transition={{ duration: 0.2, delay: Math.random() * 0.1 }}
                      >
                        <CardPreview
                          card={card}
                          size="small"
                          showQuantity={getCollectionQuantity(card.id)}
                          showPrice={true}
                          onClick={() => setSelectedCard(card)}
                          variant="levitate"
                        />
                      </motion.div>
                    ))}
                  </motion.div>
                ) : (
                  <motion.div
                    key="list"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="space-y-2"
                  >
                    {filteredResults.map((card: any) => (
                      <motion.div
                        key={card.id}
                        initial={{ opacity: 0, x: -20 }}
                        animate={{ opacity: 1, x: 0 }}
                        transition={{ duration: 0.2 }}
                      >
                        <GlassPanel variant="card" className="p-3 flex items-center gap-4" onClick={() => setSelectedCard(card)}>
                          <CardPreview card={card} size="small" showQuantity={getCollectionQuantity(card.id)} showPrice={true} variant="deck-item" />
                          <div className="flex-1 min-w-0">
                            <p className="font-medium truncate">{card.name}</p>
                            <p className="text-xs text-[var(--color-text-muted)] font-mono">
                              {card.set_name} • {card.mana_cost ? card.mana_cost.replace(/{/g, '').replace(/}/g, '') : '—'} • {card.type_line}
                            </p>
                          </div>
                          <div className="flex items-center gap-2">
                            {getCollectionQuantity(card.id) > 0 ? (
                              <Button variant="ghost" size="sm" icon={<Trash2 className="h-4 w-4" />} onClick={(e) => { e.stopPropagation(); handleRemoveFromCollection(card.id); }}>
                                Entfernen
                              </Button>
                            ) : (
                              <Button variant="primary" size="sm" icon={<Plus className="h-4 w-4" />} onClick={(e) => { e.stopPropagation(); handleAddToCollection(card); }}>
                                Hinzufügen
                              </Button>
                            )}
                          </div>
                        </GlassPanel>
                      </motion.div>
                    ))}
                  </motion.div>
                )}
              </AnimatePresence>
            )}
          </div>
        ) : (
          // Collection View
          <div>
            <div className="flex items-center justify-between mb-4">
              <h2 className="heading-2 font-serif text-xl">
                Meine Sammlung
              </h2>
              <div className="flex items-center gap-2">
                <Button variant="ghost" size="sm" icon={viewMode === 'grid' ? <ChevronDown className="h-4 w-4" /> : <ChevronUp className="h-4 w-4" />} onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')} />
              </div>
            </div>

            {collectionLoading ? (
              <GlassPanel variant="strong" className="h-64 flex items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-[var(--color-parchment)]" />
              </GlassPanel>
            ) : collection?.length === 0 ? (
              <GlassPanel variant="strong" className="h-64 flex flex-col items-center justify-center text-center gap-4">
                <Archive className="h-16 w-16 text-[var(--color-text-muted)]" />
                <div>
                  <p className="text-[var(--color-text-muted)] mb-2">Sammlung ist leer</p>
                  <p className="text-sm text-[var(--color-text-muted)] mb-4">Suche nach Karten und füge sie hinzu, oder starte einen Vollimport</p>
                  <Button variant="primary" icon={<Download className="h-4 w-4" />} onClick={handleImportAll} loading={importRunning}>
                    Vollimport starten
                  </Button>
                </div>
              </GlassPanel>
            ) : (
              <AnimatePresence mode="popLayout">
                {viewMode === 'grid' ? (
                  <motion.div
                    key="grid"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4"
                  >
                    {collection?.map((item: any) => (
                      <motion.div
                        key={item.card.id}
                        initial={{ opacity: 0, scale: 0.9, y: 20 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        transition={{ duration: 0.2, delay: Math.random() * 0.1 }}
                      >
                        <CardPreview
                          card={item.card}
                          size="small"
                          showQuantity={item.quantity}
                          showPrice={true}
                          onClick={() => setSelectedCard(item.card)}
                          variant="levitate"
                        />
                      </motion.div>
                    ))}
                  </motion.div>
                ) : (
                  <motion.div
                    key="list"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="space-y-2"
                  >
                    {collection?.map((item: any) => (
                      <motion.div
                        key={item.card.id}
                        initial={{ opacity: 0, x: -20 }}
                        animate={{ opacity: 1, x: 0 }}
                        transition={{ duration: 0.2 }}
                      >
                        <GlassPanel variant="card" className="p-3 flex items-center gap-4" onClick={() => setSelectedCard(item.card)}>
                          <CardPreview card={item.card} size="small" showQuantity={item.quantity} showPrice={true} variant="deck-item" />
                          <div className="flex-1 min-w-0">
                            <p className="font-medium truncate">{item.card.name}</p>
                            <p className="text-xs text-[var(--color-text-muted)] font-mono">
                              {item.card.set_name} • {item.condition} • x{item.quantity}
                            </p>
                          </div>
                          <div className="flex items-center gap-2">
                            <Button variant="ghost" size="sm" icon={<Edit className="h-4 w-4" />} onClick={(e) => { e.stopPropagation(); /* Edit modal */ }} />
                            <Button variant="danger" size="sm" icon={<Trash2 className="h-4 w-4" />} onClick={(e) => { e.stopPropagation(); handleRemoveFromCollection(item.card.id); }} />
                          </div>
                        </GlassPanel>
                      </motion.div>
                    ))}
                  </motion.div>
                )}
              </AnimatePresence>
            )}
          </div>
        )}

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
                <CardPreview
                  card={selectedCard}
                  size="large"
                  showPrice={true}
                  variant="levitate"
                />
                <div className="absolute top-4 right-4">
                  <Button variant="glass" size="sm" icon={<X className="h-4 w-4" />} onClick={() => setSelectedCard(null)} />
                </div>
              </motion.div>
            </motion.div>
          )}
        </AnimatePresence>
      </main>
    </div>
  );
}