import { GlassPanel } from '../components/ui/GlassPanel';
import { Button } from '../components/ui/Button';
import { Database, Download, Upload, Trash2, Palette, Shield, Info, ChevronRight } from 'lucide-react';

export default function SettingsPage() {
  return (
    <div className="min-h-screen pb-24">
      <header className="sticky top-0 z-40 bg-[var(--color-bg-base)]/80 backdrop-blur-xl border-b border-[var(--color-wood)]/10">
        <div className="max-w-7xl mx-auto px-4 md:px-8 py-4">
          <h1 className="heading-1 font-serif text-2xl md:text-3xl">Einstellungen</h1>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 md:px-8 py-8 space-y-8">
        {/* Database Section */}
        <GlassPanel variant="strong" className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-[var(--radius-subtle)] bg-[var(--color-wood)]/20 text-[var(--color-wood)]">
                <Database className="h-5 w-5" />
              </div>
              <h2 className="font-serif text-xl font-semibold">Datenbank & Daten</h2>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Datenbank zurücksetzen</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Löscht alle lokalen Daten (Sammlung, Decks, Lore-Einträge). Diese Aktion kann nicht rückgängig gemacht werden.</p>
              </div>
              <Button variant="danger" icon={<Trash2 className="h-4 w-4" />}>
                Datenbank zurücksetzen
              </Button>
            </div>

            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Daten exportieren</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Exportiert alle Daten als JSON-Datei (Sammlung, Decks, Lore).</p>
              </div>
              <Button variant="secondary" icon={<Download className="h-4 w-4" />}>
                Exportieren
              </Button>
            </div>

            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Daten importieren</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Importiert Daten aus einer JSON-Datei. Existierende Daten werden zusammengeführt.</p>
              </div>
              <Button variant="secondary" icon={<Upload className="h-4 w-4" />}>
                Importieren
              </Button>
            </div>
          </div>
        </GlassPanel>

        {/* Appearance Section */}
        <GlassPanel variant="strong" className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-[var(--radius-subtle)] bg-[var(--color-parchment)]/20 text-[var(--color-parchment)]">
                <Palette className="h-5 w-5" />
              </div>
              <h2 className="font-serif text-xl font-semibold">Darstellung</h2>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Dunkler Modus</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Verwendet ein dunkles Farbschema für die Benutzeroberfläche.</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked />
                <div className="w-11 h-6 bg-[var(--color-wood)]/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-[var(--color-wood)]/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-parchment)]"></div>
              </label>
            </div>

            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Animationen reduzieren</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Deaktiviert die meisten Animationen und Übergänge für bessere Performance.</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" />
                <div className="w-11 h-6 bg-[var(--color-wood)]/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-[var(--color-wood)]/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-parchment)]"></div>
              </label>
            </div>

            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Glas-Effekte</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Aktiviert Glassmorphism-Effekte (Backdrop-Blur, Transparenz).</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked />
                <div className="w-11 h-6 bg-[var(--color-wood)]/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-[var(--color-wood)]/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-parchment)]"></div>
              </label>
            </div>
          </div>
        </GlassPanel>

        {/* Privacy & Security Section */}
        <GlassPanel variant="strong" className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-[var(--radius-subtle)] bg-[var(--color-crimson)]/20 text-[var(--color-crimson)]">
                <Shield className="h-5 w-5" />
              </div>
              <h2 className="font-serif text-xl font-semibold">Datenschutz & Sicherheit</h2>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Lokale-First Modus</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Alle Daten bleiben lokal auf deinem Gerät. Keine Cloud-Synchronisation, kein Tracking.</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked disabled />
                <div className="w-11 h-6 bg-[var(--color-parchment)] peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-[var(--color-parchment)]/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
              </label>
            </div>

            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <div>
                <h3 className="font-medium text-[var(--color-text-main)]">Telemetrie deaktivieren</h3>
                <p className="text-sm text-[var(--color-text-muted)] mt-1">Keine anonymen Nutzungsdaten an die Entwickler senden.</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked />
                <div className="w-11 h-6 bg-[var(--color-wood)]/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-[var(--color-wood)]/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-parchment)]"></div>
              </label>
            </div>
          </div>
        </GlassPanel>

        {/* About Section */}
        <GlassPanel variant="strong" className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-[var(--radius-subtle)] bg-[var(--color-leather)]/20 text-[var(--color-leather)]">
                <Info className="h-5 w-5" />
              </div>
              <h2 className="font-serif text-xl font-semibold">Über MTG Multiverse Studio</h2>
            </div>
          </div>

          <div className="space-y-4 text-sm text-[var(--color-text-muted)]">
            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <span>Version</span>
              <span className="font-mono text-[var(--color-text-main)]">0.1.1</span>
            </div>
            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <span>Lizenz</span>
              <span className="font-mono text-[var(--color-text-main)]">GPL-3.0 (Fan Project)</span>
            </div>
            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <span>Datenquellen</span>
              <span className="font-mono text-[var(--color-text-main)]">Scryfall API, MTGJSON</span>
            </div>
            <div className="flex items-center justify-between p-4 bg-[var(--color-bg-elevated)] rounded-[var(--radius-subtle)] border border-[var(--color-wood)]/10">
              <span>Tech Stack</span>
              <span className="font-mono text-[var(--color-text-main)]">Tauri v2 + React + TypeScript + SQLite</span>
            </div>
          </div>

          <div className="mt-6 pt-6 border-t border-[var(--color-wood)]/10">
            <p className="text-sm text-[var(--color-text-muted)] text-center mb-4">
              MTG Multiverse Studio ist ein Fan-Projekt und nicht mit Wizards of the Coast verbunden.
              <br />
              Magic: The Gathering ist eine Marke von Wizards of the Coast LLC.
            </p>
            <div className="flex items-center justify-center gap-4">
              <a href="https://github.com" target="_blank" rel="noopener noreferrer" className="flex items-center gap-2 text-[var(--color-text-muted)] hover:text-[var(--color-parchment)] transition-colors">
                <span className="font-mono text-xs">GitHub</span>
                <ChevronRight className="h-4 w-4" />
              </a>
              <a href="https://scryfall.com" target="_blank" rel="noopener noreferrer" className="flex items-center gap-2 text-[var(--color-text-muted)] hover:text-[var(--color-parchment)] transition-colors">
                <span className="font-mono text-xs">Scryfall API</span>
                <ChevronRight className="h-4 w-4" />
              </a>
              <a href="https://mtgjson.com" target="_blank" rel="noopener noreferrer" className="flex items-center gap-2 text-[var(--color-text-muted)] hover:text-[var(--color-parchment)] transition-colors">
                <span className="font-mono text-xs">MTGJSON</span>
                <ChevronRight className="h-4 w-4" />
              </a>
            </div>
          </div>
        </GlassPanel>
      </main>
    </div>
  );
}