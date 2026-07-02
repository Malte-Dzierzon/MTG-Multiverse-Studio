import { GlassPanel } from '../components/ui/GlassPanel';
import { Button } from '../components/ui/Button';
import { CardPreview, DeckStackPreview } from '../components/ui/CardPreview';
import { Sparkles, Archive, FlaskConical, BookOpen, Settings, ArrowRight, Search, Plus, Shuffle } from 'lucide-react';
import { TABS } from '../types';
import { useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';

export default function HubPage() {
  const navigate = useNavigate();

  const features = [
    {
      id: 'collection',
      icon: Archive,
      title: 'Artbook & Sammlung',
      description: 'Verwalte deine gesamte Kartensammlung mit Preisen, Bedingungen und Notizen. Importiere Sets von Scryfall und MTGJSON.',
      color: 'leather',
      shortcut: '1',
    },
    {
      id: 'deckbuilder',
      icon: FlaskConical,
      title: 'Deck-Labor',
      description: 'Baue Decks mit Mana-Kurven-Analyse, Goldfishing-Simulation und Format-Legality-Check. Teste deine Decks vor dem Turnier.',
      color: 'crimson',
      shortcut: '2',
    },
    {
      id: 'lore',
      icon: BookOpen,
      title: 'Lore-Atlas',
      description: 'Erkunde die Magic-Lore mit verknüpften Einträgen zu Planeswalkern, Ebenen, Artefakten und Story-Arcs.',
      color: 'wood',
      shortcut: '3',
    },
  ];

  const quickActions = [
    { label: 'Karte suchen', icon: Search, action: () => navigate('/collection'), primary: true },
    { label: 'Neues Deck', icon: Plus, action: () => navigate('/deckbuilder'), primary: false },
    { label: 'Zufällige Karte', icon: Shuffle, action: () => navigate('/collection?random=1'), primary: false },
  ];

  return (
    <div className="min-h-screen">
      {/* Hero Section */}
      <section className="relative pt-20 pb-16 px-4 md:px-8 lg:px-16">
        <div className="max-w-7xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
            className="text-center mb-16"
          >
            <div className="inline-flex items-center gap-2 glass-panel px-4 py-2 rounded-full mb-6">
              <Sparkles className="h-5 w-5 text-[var(--color-parchment)]" />
              <span className="font-mono text-xs text-[var(--color-parchment)] tracking-wider">
                MTG Multiverse Studio v0.1
              </span>
            </div>
            <h1 className="heading-1 font-serif mb-4">
              Dein Tor zum <span className="text-[var(--color-parchment)]">Multiversum</span>
            </h1>
            <p className="text-[var(--color-text-muted)] text-lg max-w-2xl mx-auto">
              Verwalte deine Sammlung, baue wettbewerbsfähige Decks und erforsche die Lore — 
              alles in einer App mit lokaler KI-Unterstützung.
            </p>
          </motion.div>

          {/* Quick Actions */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1], delay: 0.2 }}
            className="flex flex-wrap items-center justify-center gap-3 mb-16"
          >
            {quickActions.map((action, index) => (
              <motion.button
                key={action.label}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1], delay: 0.3 + index * 0.1 }}
                onClick={action.action}
                className={`
                  flex items-center gap-2 px-5 py-3 rounded-[var(--radius-subtle)]
                  font-medium transition-fast focus-ring
                  ${action.primary 
                    ? 'bg-[var(--color-parchment)] text-[var(--color-bg-base)] hover:bg-[var(--color-leather)]'
                    : 'glass-panel text-[var(--color-text-main)] hover:bg-[var(--color-bg-interactive)]'
                  }
                `}
              >
                <action.icon className="h-4 w-4" />
                {action.label}
              </motion.button>
            ))}
          </motion.div>
        </div>
      </section>

      {/* Feature Cards */}
      <section className="px-4 md:px-8 lg:px-16 pb-20">
        <div className="max-w-7xl mx-auto">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {features.map((feature, index) => (
              <motion.div
                key={feature.id}
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1], delay: 0.3 + index * 0.1 }}
                className="group"
              >
                <GlassPanel 
                  variant="card" 
                  hover 
                  onClick={() => navigate(TABS.find(t => t.id === feature.id)?.route || '/')}
                  className="h-full flex flex-col p-6"
                >
                  <div className="flex items-start justify-between mb-4">
                    <div className={`
                      p-3 rounded-[var(--radius-subtle)]
                      bg-[var(--color-${feature.color})]/20
                      text-[var(--color-${feature.color})]
                      group-hover:bg-[var(--color-${feature.color})]/40
                      transition-colors
                    `}>
                      <feature.icon className="h-7 w-7" />
                    </div>
                    <kbd className="font-mono text-[10px] px-2 py-1 rounded bg-[var(--color-bg-surface)]/50 border border-[var(--color-wood)]/20 text-[var(--color-text-muted)]">
                      {feature.shortcut}
                    </kbd>
                  </div>
                  <h3 className="font-serif text-xl font-semibold mb-2 text-[var(--color-text-main)]">
                    {feature.title}
                  </h3>
                  <p className="text-[var(--color-text-muted)] text-sm mb-6 flex-1">
                    {feature.description}
                  </p>
                  <Button variant="ghost" fullWidth icon={ArrowRight} iconPosition="right">
                    Erkunden
                  </Button>
                </GlassPanel>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Sample Deck Stack Preview (placeholder) */}
      <section className="px-4 md:px-8 lg:px-16 pb-20">
        <div className="max-w-7xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1], delay: 0.6 }}
          >
            <div className="flex items-center justify-between mb-6">
              <div>
                <h2 className="heading-2 font-serif">Letzte Decks</h2>
                <p className="text-[var(--color-text-muted)] text-sm mt-1">Deine kürzlich bearbeiteten Decklisten</p>
              </div>
              <Button variant="glass" icon={Plus} iconPosition="right" onClick={() => navigate('/deckbuilder')}>
                Neues Deck
              </Button>
            </div>
            <GlassPanel variant="strong" className="p-6">
              <div className="flex items-center justify-center h-64">
                <div className="text-center text-[var(--color-text-muted)]">
                  <FlaskConical className="h-12 w-12 mx-auto mb-4 opacity-30" />
                  <p className="font-mono text-sm">Noch keine Decks erstellt</p>
                  <p className="text-xs mt-1">Starte dein erstes Deck im Deck-Labor</p>
                </div>
              </div>
            </GlassPanel>
          </motion.div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-[var(--color-wood)]/10 py-8 px-4">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4 text-[var(--color-text-muted)] text-sm">
          <p className="font-mono">MTG Multiverse Studio — Fan Project unter GPL-v3</p>
          <div className="flex items-center gap-4">
            <a href="https://scryfall.com" target="_blank" rel="noopener" className="hover:text-[var(--color-parchment)] transition-colors">
              Daten von Scryfall
            </a>
            <a href="https://mtgjson.com" target="_blank" rel="noopener" className="hover:text-[var(--color-parchment)] transition-colors">
              MTGJSON
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}