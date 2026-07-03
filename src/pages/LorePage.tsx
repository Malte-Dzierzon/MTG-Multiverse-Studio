import { useState } from 'react';
import { GlassPanel } from '../components/ui/GlassPanel';
import { Button } from '../components/ui/Button';
import { CardPreview } from '../components/ui/CardPreview';
import { SearchInput, CommandPaletteTrigger } from '../components/ui/SearchInput';
import { Sparkles, Plus, Star, Map as MapIcon, Gem, Users, Book, Zap, Castle } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { loadLoreEntries, searchLore, createLoreEntry } from '../services/api';
import { useQuery } from '@tanstack/react-query';
import { LoreEntry } from '../types';

const LORE_TYPES = [
  { id: 'all', label: 'Alle', icon: Sparkles },
  { id: 'planeswalker', label: 'Planeswalker', icon: Star },
  { id: 'plane', label: 'Ebenen', icon: MapIcon },
  { id: 'artifact', label: 'Artefakte', icon: Gem },
  { id: 'faction', label: 'Fraktionen', icon: Users },
  { id: 'story', label: 'Story-Arcs', icon: Book },
  { id: 'event', label: 'Ereignisse', icon: Zap },
  { id: 'location', label: 'Orte', icon: Castle },
];

const ICON_COMPONENTS: Record<string, React.ComponentType<{ className?: string }>> = {
  Star, MapIcon, Gem, Users, Book, Zap, Castle, Sparkles,
};

function getTypeColor(type: string): string {
  const colors: Record<string, string> = {
    planeswalker: 'parchment',
    plane: 'wood',
    artifact: 'gold',
    faction: 'crimson',
    story: 'leather',
    event: 'parchment',
    location: 'wood',
  };
  return colors[type] || 'muted';
}

function renderIcon(type: string, className: string = 'h-5 w-5') {
  const IconComponent = ICON_COMPONENTS[type];
  if (IconComponent) {
    return <IconComponent className={className} />;
  }
  return <Sparkles className={className} />;
}

export default function LorePage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedType, setSelectedType] = useState('all');
  const [selectedEntry, setSelectedEntry] = useState<LoreEntry | null>(null);
  const [showCreate, setShowCreate] = useState(false);

  const { data: entries, isLoading } = useQuery({
    queryKey: ['lore', selectedType, searchQuery],
    queryFn: () => {
      if (searchQuery) return searchLore({ query: searchQuery });
      return loadLoreEntries({ lore_type: selectedType === 'all' ? undefined : selectedType });
    },
  });

  return (
    <div className="min-h-screen pb-24">
      {/* Header */}
      <header className="sticky top-0 z-40 bg-[var(--color-bg-base)]/80 backdrop-blur-xl border-b border-[var(--color-wood)]/10">
        <div className="max-w-7xl mx-auto px-4 md:px-8 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <h1 className="heading-1 font-serif text-2xl md:text-3xl">Lore-Atlas</h1>
            </div>
            <div className="flex items-center gap-2">
              <CommandPaletteTrigger onOpen={() => {}} />
              <Button variant="secondary" size="sm" icon={<Plus className="h-4 w-4" />} onClick={() => setShowCreate(true)}>
                Neuer Eintrag
              </Button>
            </div>
          </div>
        </div>
        
        {/* Type Filter Tabs */}
        <div className="px-4 md:px-8 pb-4">
          <div className="flex gap-2 overflow-x-auto pb-2 -mb-2 scrollbar-hide">
            {LORE_TYPES.map((type) => (
              <Button
                key={type.id}
                variant={selectedType === type.id ? 'primary' : 'ghost'}
                size="sm"
                icon={<type.icon className="h-3 w-3" />}
                onClick={() => setSelectedType(type.id)}
              >
                {type.label}
              </Button>
            ))}
          </div>
        </div>
      </header>

      {/* Search Bar */}
      <div className="px-4 md:px-8 pb-6">
        <SearchInput
          value={searchQuery}
          onChange={setSearchQuery}
          placeholder="Lore durchsuchen (Planeswalker, Ebenen, Artefakte, Story-Arcs)..."
          showShortcut
        />
      </div>

      {/* Content */}
      <main className="max-w-7xl mx-auto px-4 md:px-8 pb-12">
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <div className="animate-spin h-8 w-8 border-2 border-[var(--color-parchment)] border-t-transparent rounded-full" />
          </div>
        ) : (
          <AnimatePresence mode="wait">
            <motion.div
              key={selectedType + searchQuery}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.3 }}
            >
              {entries && entries.length > 0 ? (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {entries.map((entry: LoreEntry, index: number) => (
                    <motion.div
                      key={entry.id}
                      initial={{ opacity: 0, scale: 0.95 }}
                      animate={{ opacity: 1, scale: 1 }}
                      transition={{ duration: 0.3, delay: index * 0.05 }}
                    >
                      <GlassPanel
                        variant="card"
                        hover
                        className="p-5 h-full flex flex-col"
                        onClick={() => setSelectedEntry(entry)}
                      >
                        <div className="flex items-start justify-between mb-3">
                          <div className={`p-2 rounded-[var(--radius-subtle)] bg-[var(--color-${getTypeColor(entry.lore_type)})]/20 text-[var(--color-${getTypeColor(entry.lore_type)})]`}>
                                                      {renderIcon(entry.lore_type, 'h-5 w-5')}
                                                    </div>
                          <span className="font-mono text-xs text-[var(--color-text-muted)]">#{entry.id}</span>
                        </div>
                        <h3 className="font-serif text-lg font-semibold mb-2 text-[var(--color-text-main)] line-clamp-1">
                          {entry.title}
                        </h3>
                        <p className="text-[var(--color-text-muted)] text-sm mb-4 flex-1 line-clamp-3">
                          {entry.content.slice(0, 150)}...
                        </p>
                        <div className="flex flex-wrap gap-1.5">
                          {entry.related_cards?.slice(0, 3).map((cardId: string) => (
                            <span key={cardId} className="px-2 py-0.5 text-xs font-mono bg-[var(--color-bg-interactive)] rounded border border-[var(--color-wood)]/30 text-[var(--color-text-muted)]">
                              {cardId}
                            </span>
                          ))}
                          {entry.related_cards && entry.related_cards.length > 3 && (
                            <span className="px-2 py-0.5 text-xs font-mono bg-[var(--color-bg-interactive)] rounded border border-[var(--color-wood)]/30 text-[var(--color-text-muted)]">
                              +{entry.related_cards.length - 3}
                            </span>
                          )}
                        </div>
                      </GlassPanel>
                    </motion.div>
                  ))}
                </div>
              ) : (
                <GlassPanel variant="strong" className="p-12 text-center">
                  <Sparkles className="h-12 w-12 mx-auto mb-4 opacity-30" />
                  <p className="text-[var(--color-text-muted)]">
                    {searchQuery ? 'Keine Ergebnisse für deine Suche.' : 'Keine Lore-Einträge gefunden.'}
                  </p>
                  <Button variant="secondary" className="mt-4" onClick={() => { setSearchQuery(''); setSelectedType('all'); }}>
                    Filter zurücksetzen
                  </Button>
                </GlassPanel>
              )}
            </motion.div>
          </AnimatePresence>
        )}
      </main>

      {/* Entry Detail Modal */}
      <AnimatePresence>
        {selectedEntry && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
            onClick={() => setSelectedEntry(null)}
          >
            <motion.div
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative max-w-3xl w-full max-h-[90vh] overflow-y-auto bg-[var(--color-bg-surface)] rounded-[var(--radius-strong)]"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="p-6">
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <div className="flex items-center gap-2 mb-2">
                      <span className={`px-2 py-0.5 rounded font-mono text-xs bg-[var(--color-${getTypeColor(selectedEntry.lore_type)})]/20 text-[var(--color-${getTypeColor(selectedEntry.lore_type)})]`}>
                        {LORE_TYPES.find(t => t.id === selectedEntry.lore_type)?.label || selectedEntry.lore_type}
                      </span>
                      <span className="font-mono text-xs text-[var(--color-text-muted)]">#{selectedEntry.id}</span>
                    </div>
                    <h2 className="heading-2 font-serif text-2xl md:text-3xl">{selectedEntry.title}</h2>
                  </div>
                  <Button variant="glass" size="sm" icon={<span className="h-4 w-4">✕</span>} onClick={() => setSelectedEntry(null)} />
                </div>

                <div className="prose prose-invert max-w-none text-[var(--color-text-main)]">
                  <p className="whitespace-pre-wrap">{selectedEntry.content}</p>
                </div>

                {selectedEntry.related_cards && selectedEntry.related_cards.length > 0 && (
                  <div className="mt-6">
                    <h3 className="font-medium mb-3">Verknüpfte Karten</h3>
                    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                      {selectedEntry.related_cards.slice(0, 8).map((cardId: string) => (
                        <CardPreview key={cardId} card={{ id: cardId, name: cardId } as any} size="small" variant="default" />
                      ))}
                    </div>
                  </div>
                )}

                {selectedEntry.metadata && Object.keys(selectedEntry.metadata).length > 0 && (
                  <div className="mt-6 pt-6 border-t border-[var(--color-wood)]/20">
                    <h3 className="font-medium mb-3">Metadaten</h3>
                    <div className="grid grid-cols-2 gap-2 text-sm">
                      {Object.entries(selectedEntry.metadata).map(([key, value]) => (
                        <div key={key} className="flex justify-between">
                          <span className="text-[var(--color-text-muted)]">{key}:</span>
                          <span>{String(value)}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Create Entry Modal */}
      <AnimatePresence>
        {showCreate && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
            onClick={() => setShowCreate(false)}
          >
            <motion.div
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative max-w-2xl w-full max-h-[90vh] overflow-y-auto bg-[var(--color-bg-surface)] rounded-[var(--radius-strong)]"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="p-6">
                <div className="flex items-center justify-between mb-6">
                  <h2 className="font-serif text-2xl font-semibold">Neuer Lore-Eintrag</h2>
                  <Button variant="glass" size="sm" icon={<span className="h-4 w-4">✕</span>} onClick={() => setShowCreate(false)} />
                </div>
                <CreateLoreForm onClose={() => setShowCreate(false)} />
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function CreateLoreForm({ onClose }: { onClose: () => void }) {
  const [formData, setFormData] = useState({
    title: '',
    lore_type: 'planeswalker',
    content: '',
    related_cards: '',
    metadata: '',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await createLoreEntry(
        formData.title,
        formData.lore_type,
        formData.content,
        formData.related_cards.split(',').map(s => s.trim()).filter(Boolean),
        formData.metadata ? JSON.parse(formData.metadata) : {}
      );
      onClose();
    } catch (error) {
      console.error('Failed to create lore entry:', error);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-1">Titel</label>
        <input
          type="text"
          value={formData.title}
          onChange={(e) => setFormData({ ...formData, title: e.target.value })}
          className="w-full glass-panel px-3 py-2 rounded border border-[var(--color-wood)]/20 bg-transparent text-[var(--color-text-main)] focus:border-[var(--color-parchment)] focus:outline-none"
          required
        />
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Typ</label>
        <select
          value={formData.lore_type}
          onChange={(e) => setFormData({ ...formData, lore_type: e.target.value })}
          className="w-full glass-panel px-3 py-2 rounded border border-[var(--color-wood)]/20 bg-transparent text-[var(--color-text-main)] focus:border-[var(--color-parchment)] focus:outline-none"
        >
          {LORE_TYPES.filter(t => t.id !== 'all').map((type) => (
            <option key={type.id} value={type.id}>{type.label}</option>
          ))}
        </select>
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Inhalt</label>
        <textarea
          value={formData.content}
          onChange={(e) => setFormData({ ...formData, content: e.target.value })}
          rows={8}
          className="w-full glass-panel px-3 py-2 rounded border border-[var(--color-wood)]/20 bg-transparent text-[var(--color-text-main)] focus:border-[var(--color-parchment)] focus:outline-none resize-none"
          required
        />
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Verknüpfte Karten (kommagetrennt)</label>
        <input
          type="text"
          value={formData.related_cards}
          onChange={(e) => setFormData({ ...formData, related_cards: e.target.value })}
          className="w-full glass-panel px-3 py-2 rounded border border-[var(--color-wood)]/20 bg-transparent text-[var(--color-text-main)] focus:border-[var(--color-parchment)] focus:outline-none"
          placeholder="z.B. Lightning Bolt, Serra Angel, Black Lotus"
        />
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Metadaten (JSON, optional)</label>
        <textarea
          value={formData.metadata}
          onChange={(e) => setFormData({ ...formData, metadata: e.target.value })}
          rows={4}
          className="w-full glass-panel px-3 py-2 rounded border border-[var(--color-wood)]/20 bg-transparent text-[var(--color-text-main)] focus:border-[var(--color-parchment)] focus:outline-none resize-none font-mono text-sm"
          placeholder='{ "era": "Phyrexian Invasion", "plane": "Dominaria" }'
        />
      </div>
      <div className="flex gap-3 justify-end pt-4">
        <Button type="button" variant="secondary" onClick={onClose}>
          Abbrechen
        </Button>
        <Button type="submit" variant="primary">
          Erstellen
        </Button>
      </div>
    </form>
  );
}