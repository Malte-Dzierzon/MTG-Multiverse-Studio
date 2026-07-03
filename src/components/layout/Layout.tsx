import { cn } from '../../utils/cn';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { AnimatePresence, motion } from 'framer-motion';
import { useState, useEffect, useCallback } from 'react';
import { Sparkles, Archive, FlaskConical, BookOpen, Settings, Command, X } from 'lucide-react';
import { TABS, type TabId } from '../../types';
import { GlassPanel } from '../ui/GlassPanel';
import { Button } from '../ui/Button';

export default function Layout() {
  const location = useLocation();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<TabId>('hub');
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [tabBarVisible, setTabBarVisible] = useState(true);
  const [lastScrollY, setLastScrollY] = useState(0);

  // Sync active tab with route
  useEffect(() => {
    const tab = TABS.find(t => location.pathname === t.route || location.pathname.startsWith(t.route + '/'));
    if (tab) setActiveTab(tab.id);
  }, [location.pathname]);

  // Auto-hide tab bar on scroll down
  useEffect(() => {
    const handleScroll = () => {
      const currentScrollY = window.scrollY;
      if (currentScrollY > lastScrollY && currentScrollY > 100) {
        setTabBarVisible(false);
      } else {
        setTabBarVisible(true);
      }
      setLastScrollY(currentScrollY);
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => window.removeEventListener('scroll', handleScroll);
  }, [lastScrollY]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + K for command palette
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsCommandPaletteOpen(true);
      }
      // Escape to close command palette
      if (e.key === 'Escape') {
        setIsCommandPaletteOpen(false);
      }
      // Number keys 1-5 for tabs
      if (e.key >= '1' && e.key <= '5' && !(e.metaKey || e.ctrlKey || e.altKey)) {
        const target = document.activeElement;
        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) return;
        e.preventDefault();
        const tabs = ['hub', 'collection', 'deckbuilder', 'lore', 'settings'];
        const index = parseInt(e.key, 10) - 1;
        if (tabs[index]) {
          navigate(TABS.find(t => t.id === tabs[index])?.route || '/');
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [navigate]);

  const handleTabClick = useCallback((tab: TabId) => {
    setActiveTab(tab);
    navigate(TABS.find(t => t.id === tab)?.route || '/');
  }, [navigate]);

  // Command palette actions
  const commandActions = [
    { id: 'search', label: 'Karten suchen', shortcut: '⌘K', icon: Command, action: () => navigate('/collection') },
    { id: 'new-deck', label: 'Neues Deck erstellen', shortcut: '⌘N', icon: FlaskConical, action: () => navigate('/deckbuilder') },
    { id: 'collection', label: 'Meine Sammlung', shortcut: '1', icon: Archive, action: () => navigate('/collection') },
    { id: 'deckbuilder', label: 'Deck-Labor', shortcut: '2', icon: FlaskConical, action: () => navigate('/deckbuilder') },
    { id: 'lore', label: 'Lore-Atlas', shortcut: '3', icon: BookOpen, action: () => navigate('/lore') },
    { id: 'settings', label: 'Einstellungen', shortcut: '4', icon: Settings, action: () => navigate('/settings') },
  ];

  return (
    <div className="min-h-screen bg-[var(--color-bg-base)] text-[var(--color-text-main)]">
      {/* Page content with transitions */}
      <AnimatePresence mode="wait">
        <motion.div
          key={location.pathname}
          initial={{ opacity: 0, scale: 0.98, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.98, y: -20 }}
          transition={{ duration: 0.35, ease: [0.16, 1, 0.3, 1] }}
          className="pb-20" // Space for tab bar
        >
          <Outlet />
        </motion.div>
      </AnimatePresence>

      {/* Bottom Tab Bar */}
      <AnimatePresence>
        {tabBarVisible && (
          <motion.div
            initial={{ y: 100, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 100, opacity: 0 }}
            transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
            className="fixed bottom-0 left-0 right-0 z-50 px-4 pb-4 safe-area-bottom"
          >
            <GlassPanel variant="strong" className="max-w-7xl mx-auto px-2 py-1.5">
              <div className="flex items-center justify-around">
                {TABS.map((tab) => {
                  const Icon = tab.icon ? (() => {
                    const icons: Record<string, React.ComponentType<{ className?: string }>> = {
                      sparkles: Sparkles,
                      archive: Archive,
                      'flask-conical': FlaskConical,
                      'book-open': BookOpen,
                      settings: Settings,
                    };
                    return icons[tab.icon] || Sparkles;
                  })() : Sparkles;

                  const isActive = activeTab === tab.id;
                  const colorMap: Record<string, string> = {
                    parchment: 'text-[var(--color-parchment)]',
                    leather: 'text-[var(--color-leather)]',
                    crimson: 'text-[var(--color-crimson)]',
                    wood: 'text-[var(--color-wood)]',
                    muted: 'text-[var(--color-text-muted)]',
                  };

                  return (
                    <button
                      key={tab.id}
                      onClick={() => handleTabClick(tab.id)}
                      className={cn(
                        'relative flex flex-col items-center gap-1 px-3 py-2 rounded-[var(--radius-subtle)]',
                        'transition-fast focus-ring',
                        isActive
                          ? 'bg-[var(--color-bg-interactive)]/50'
                          : 'hover:bg-[var(--color-bg-interactive)]/30'
                      )}
                      aria-current={isActive ? 'page' : undefined}
                      aria-label={tab.label}
                    >
                      <Icon 
                        className={cn(
                          'h-5 w-5 transition-colors',
                          isActive ? colorMap[tab.color] : 'text-[var(--color-text-muted)]'
                        )}
                        aria-hidden="true"
                      />
                      <span className={cn(
                        'font-mono text-[10px] font-medium transition-colors',
                        isActive ? colorMap[tab.color] : 'text-[var(--color-text-muted)]'
                      )}>
                        {tab.label}
                      </span>
                      {isActive && (
                        <motion.div
                          initial={{ scaleX: 0 }}
                          animate={{ scaleX: 1 }}
                          className="absolute bottom-0 left-1/2 -translate-x-1/2 h-0.5 w-[calc(100%-8px)] rounded-full"
                          style={{ backgroundColor: `var(--color-${tab.color})` }}
                        />
                      )}
                    </button>
                  );
                })}
              </div>
            </GlassPanel>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Command Palette */}
      <AnimatePresence>
        {isCommandPaletteOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="cmd-palette-overlay"
              onClick={() => setIsCommandPaletteOpen(false)}
            />
            <motion.div
              initial={{ opacity: 0, y: -20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              className="cmd-palette-window"
            >
              <div className="p-4">
                <div className="flex items-center gap-3 mb-4">
                  <div className="relative flex-1">
                    <Command className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-[var(--color-text-muted)]" />
                    <input
                      type="text"
                      placeholder="Befehl suchen... (oder Tab-Taste drücken)"
                      autoFocus
                      className="w-full glass-panel pl-10 pr-4 py-3 text-[var(--color-text-main)] placeholder-[var(--color-text-muted)] font-mono text-sm"
                      onKeyDown={(e) => {
                        if (e.key === 'Escape') setIsCommandPaletteOpen(false);
                      }}
                    />
                  </div>
                  <Button variant="ghost" size="sm" onClick={() => setIsCommandPaletteOpen(false)}>
                    <X className="h-4 w-4" />
                  </Button>
                </div>

                <div className="space-y-1">
                  {commandActions.map((action) => {
                    const Icon = action.icon;
                    return (
                      <button
                        key={action.id}
                        onClick={() => { action.action(); setIsCommandPaletteOpen(false); }}
                        className="w-full glass-panel px-4 py-3 flex items-center gap-3 text-left hover:bg-[var(--color-bg-interactive)]/50 transition-fast focus-ring"
                      >
                        <Icon className="h-5 w-5 text-[var(--color-text-muted)] flex-shrink-0" />
                        <span className="font-medium text-[var(--color-text-main)]">{action.label}</span>
                        <kbd className="ml-auto font-mono text-[10px] px-2 py-0.5 rounded bg-[var(--color-bg-surface)]/50 border border-[var(--color-wood)]/20 text-[var(--color-text-muted)]">
                          {action.shortcut}
                        </kbd>
                      </button>
                    );
                  })}
                </div>

                <div className="mt-4 pt-4 border-t border-[var(--color-wood)]/20">
                  <p className="font-mono text-[10px] text-[var(--color-text-muted)] text-center">
                    Tipp: Drücke 1–5 für direkte Navigation zu den Tabs
                  </p>
                </div>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}