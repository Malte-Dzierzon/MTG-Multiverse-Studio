import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { GlassPanel } from '../components/ui/GlassPanel';
import { Button } from '../components/ui/Button';
import { CardPreview } from '../components/ui/CardPreview';
import { ChevronLeft, Map, Star, Gem, Users, Book, Zap, Castle, Sparkles } from 'lucide-react';
import { getLoreEntry, searchLore } from '../services/api';
import { LoreEntry } from '../types';
import { motion, AnimatePresence } from 'framer-motion';

const LORE_TYPES = [
  { id: 'planeswalker', label: 'Planeswalker', icon: Star },
  { id: 'plane', label: 'Ebenen', icon: Map },
  { id: 'artifact', label: 'Artefakte', icon: Gem },
  { id: 'faction', label: 'Fraktionen', icon: Users },
  { id: 'story', label: 'Story-Arcs', icon: Book },
  { id: 'event', label: 'Ereignisse', icon: Zap },
  { id: 'location', label: 'Orte', icon: Castle },
];

const ICON_COMPONENTS: Record<string, React.ComponentType<{ className?: string }>> = {
  Star, Map, Gem, Users, Book, Zap, Castle, Sparkles,
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

export default function LoreDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [relatedCard, setRelatedCard] = useState<any>(null);

  const { data: response, isLoading, error } = useQuery({
    queryKey: ['loreEntry', id],
    queryFn: () => getLoreEntry(Number(id)),
    enabled: !!id,
  });

  const entry = response?.entry;

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin h-8 w-8 border-2 border-[var(--color-parchment)] border-t-transparent rounded-full" />
      </div>
    );
  }

  if (error || !entry) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <GlassPanel variant="strong" className="p-8 text-center">
          <Sparkles className="h-12 w-12 mx-auto mb-4 opacity-30" />
          <p className="text-[var(--color-text-muted)]">Lore-Eintrag nicht gefunden</p>
          <Button onClick={() => navigate('/lore')} className="mt-4">
            Zurück zum Lore-Atlas
          </Button>
        </GlassPanel>
      </div>
    );
  }

  const typeInfo = LORE_TYPES.find(t => t.id === entry.lore_type);

  return (
    <div className="min-h-screen pb-24">
      {/* Header */}
      <header className="sticky top-0 z-40 bg-[var(--color-bg-base)]/80 backdrop-blur-xl border-b border-[var(--color-wood)]/10">
        <div className="max-w-4xl mx-auto px-4 md:px-8 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Button variant="ghost" size="sm" icon={<ChevronLeft className="h-4 w-4" />} onClick={() => navigate('/lore')} />
              <div>
                <h1 className="heading-1 font-serif text-2xl md:text-3xl">{entry.title}</h1>
                <p className="text-[var(--color-text-muted)] text-sm font-mono">#{entry.id}</p>
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Content */}
      <main className="max-w-4xl mx-auto px-4 md:px-8 py-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
        >
          {/* Type Badge */}
          <div className="flex items-center gap-3 mb-6">
            <span className={`px-3 py-1 rounded-full font-mono text-xs bg-[var(--color-${getTypeColor(entry.lore_type)})]/20 text-[var(--color-${getTypeColor(entry.lore_type)})]`}>
              {typeInfo?.label || entry.lore_type}
            </span>
          </div>

          {/* Main Content */}
          <GlassPanel variant="strong" className="p-8">
            <div className="prose prose-invert max-w-none text-[var(--color-text-main)]">
              <p className="whitespace-pre-wrap">{entry.content}</p>
            </div>

            {/* Related Cards */}
            {entry.related_cards && entry.related_cards.length > 0 && (
              <div className="mt-8 pt-8 border-t border-[var(--color-wood)]/20">
                <h2 className="font-serif text-xl font-semibold mb-4">Verknüpfte Karten ({entry.related_cards.length})</h2>
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                  {entry.related_cards.slice(0, 12).map((cardId: string) => (
                    <CardPreview
                      key={cardId}
                      card={{ id: cardId, name: cardId } as any}
                      size="small"
                      variant="default"
                      onClick={() => setRelatedCard({ id: cardId, name: cardId })}
                    />
                  ))}
                  {entry.related_cards.length > 12 && (
                    <div className="col-span-full flex items-center justify-center">
                      <span className="text-[var(--color-text-muted)] text-sm">
                        +{entry.related_cards.length - 12} weitere Karten
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Metadata */}
            {entry.metadata && Object.keys(entry.metadata).length > 0 && (
              <div className="mt-8 pt-8 border-t border-[var(--color-wood)]/20">
                <h2 className="font-serif text-xl font-semibold mb-4">Metadaten</h2>
                <div className="grid grid-cols-2 gap-3 text-sm">
                  {Object.entries(entry.metadata).map(([key, value]) => (
                    <div key={key} className="flex justify-between">
                      <span className="text-[var(--color-text-muted)]">{key}:</span>
                      <span>{String(value)}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </GlassPanel>

          {/* Related Entries Search */}
          <div className="mt-8">
            <h2 className="font-serif text-xl font-semibold mb-4">Ähnliche Lore-Einträge</h2>
            <RelatedLoreEntries searchTerm={entry.title} excludeId={entry.id} />
          </div>
        </motion.div>
      </main>

      {/* Card Detail Modal */}
      <AnimatePresence>
        {relatedCard && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
            onClick={() => setRelatedCard(null)}
          >
            <motion.div
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative max-w-md w-full max-h-[90vh] overflow-y-auto"
              onClick={(e) => e.stopPropagation()}
            >
              <CardPreview card={relatedCard} size="large" showPrice={true} variant="levitate" />
              <div className="absolute top-4 right-4">
                <Button variant="glass" size="sm" icon={<span className="h-4 w-4">✕</span>} onClick={() => setRelatedCard(null)} />
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function RelatedLoreEntries({ searchTerm, excludeId }: { searchTerm: string; excludeId: number }) {
  const { data: entries } = useQuery({
    queryKey: ['lore', 'related', searchTerm],
    queryFn: () => searchLore({ query: searchTerm.split(' ')[0] }),
  });

  if (!entries || entries.length === 0) return null;

  const filtered = entries.filter((e: LoreEntry) => e.id !== excludeId).slice(0, 3);

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {filtered.map((entry: LoreEntry) => (
        <GlassPanel key={entry.id} variant="card" hover className="p-5 h-full flex flex-col">
          <div className="flex items-start justify-between mb-3">
            <div className={`p-2 rounded-[var(--radius-subtle)] bg-[var(--color-${getTypeColor(entry.lore_type)})]/20 text-[var(--color-${getTypeColor(entry.lore_type)})]`}>
              {renderIcon(entry.lore_type, 'h-5 w-5')}
            </div>
            <span className="font-mono text-xs text-[var(--color-text-muted)]">#{entry.id}</span>
          </div>
          <h3 className="font-serif text-lg font-semibold mb-2 text-[var(--color-text-main)] line-clamp-1">
            {entry.title}
          </h3>
          <p className="text-[var(--color-text-muted)] text-sm mb-4 flex-1 line-clamp-2">
            {entry.content.slice(0, 100)}...
          </p>
        </GlassPanel>
      ))}
    </div>
  );
}