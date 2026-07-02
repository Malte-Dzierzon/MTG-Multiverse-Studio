# MTG Multiverse Studio — Frontend-Architektur & Design-Plan

> **Version:** 1.0 — Juli 2026  
> **Fokus:** Phase 2 — Frontend-Grundgerüst (React + TypeScript + Tailwind)  
> **Status:** Entwurf zur Diskussion

---

## 1. Gesamtarchitektur

### 1.1 Verzeichnisstruktur (Ziel)

```
src/
├── App.tsx                        ← Root: Router + Layout-Shell
├── main.tsx                       ← ReactDOM.createRoot + Provider-Wrapper
├── index.css                      ← Tailwind-Direktiven + globale Basisstyles (aus test.html)
│
├── layouts/
│   └── AppShell.tsx               ← Sidebar + Content-Area + AnimatedOutlet
│   └── AnimatedOutlet.tsx         ← Page-Transition-Wrapper (Framer Motion)
│
├── components/
│   ├── ui/                        ← Primitive, wiederverwendbare UI-Elemente
│   │   ├── Button.tsx
│   │   ├── GlassPanel.tsx         ← Das "spec-panel" aus test.html als Komponente
│   │   ├── CardPreview.tsx        ← MTG-Kartenvorschau
│   │   ├── SearchInput.tsx
│   │   ├── StatusDot.tsx
│   │   ├── Modal.tsx
│   │   ├── LoadingSpinner.tsx
│   │   └── Tooltip.tsx
│   │
│   ├── layout/                    ← Layout-Komponenten
│   │   ├── Sidebar.tsx            ← "Die Planarpforte" – Navigation
│   │   ├── SidebarItem.tsx        ← Einzelnavigationselement
│   │   └── PageHeader.tsx         ← Seitentitel + Action-Buttons
│   │
│   └── features/                  ← Feature-spezifische Komponenten
│       ├── collection/
│       │   ├── CardGrid.tsx
│       │   ├── CollectionFilters.tsx
│       │   └── CollectionStatsBar.tsx
│       ├── deckbuilder/
│       │   ├── DeckList.tsx
│       │   ├── ManaCurveChart.tsx
│       │   └── CardSearchPanel.tsx
│       └── lore/
│           ├── LoreTimeline.tsx
│           ├── StoryReader.tsx
│           └── PlaneExplorer.tsx
│
├── pages/
│   ├── CollectionPage.tsx         ← /collection
│   ├── DeckbuilderPage.tsx        ← /deckbuilder (/deck/:id)
│   ├── DeckDetailPage.tsx         ← /deck/:id
│   ├── LoreAtlasPage.tsx          ← /lore
│   ├── LoreDetailPage.tsx         ← /lore/:id
│   ├── HubPage.tsx                ← / (optional: Landing/Hub)
│   └── SettingsPage.tsx           ← /settings (zukünftig)
│
├── hooks/
│   ├── useCards.ts                ← searchCards, getCard
│   ├── useDeck.ts                 ← createDeck, getDeck, listDecks
│   ├── useCollection.ts           ← getCollection, addToCollection
│   ├── useLore.ts                 ← loadLoreEntries
│   └── usePageTransition.ts       ← Router-Animationen-Utilities
│
├── store/
│   ├── appStore.ts                ← Globaler Zustand (Seite, Theme, etc.)
│   ├── collectionStore.ts         ← Collection-State
│   └── deckStore.ts               ← Deck-Editor-State
│
├── services/
│   ├── api.ts                     ← Alle Tauri invoke()-Wrapper (+ frisch aus commands.rs generierte Typen)
│   ├── card_service.ts            ← Card-Spezifische Logik
│   └── scryfall_client.ts         ← (wird über Backend geroutet, dient als Referenz)
│
├── types/
│   └── index.ts                   ← TypeScript-Typen für CardResponse, DeckResponse, etc.
│
├── utils/
│   ├── constants.ts               ← Design-Token-Konstanten, URLs, etc.
│   ├── helpers.ts                 ← Formatierungs-Helfer (Mana-Symbole, Preise, etc.)
│   └── cn.ts                      ← clsx + tailwind-merge Utility
│
└── assets/
    ├── icons/                     ← SVG-Icons für Sidebar
    └── fonts/                     ← Lokale Font-Files (Fallback)
```

### 1.2 Libraries (package.json Updates)

| Package | Zweck | Version |
|---------|-------|---------|
| `react-router-dom` | Routing | ^6.x ✅ (vorhanden) |
| `@tauri-apps/api` | Tauri IPC (invoke) | ^2.x |
| `zustand` | State Management | ^5.x |
| `tailwindcss` | Utility-first CSS | ^4.x |
| `@tailwindcss/vite` | Tailwind v4 Vite Plugin | ^4.x |
| `framer-motion` | Animationen, Page Transitions | ^12.x |
| `lucide-react` | Icons (Magic-kompatible Icons) | ^0.x |
| `clsx` + `tailwind-merge` | Klassen-Kombination | ^2.x |
| `@tanstack/react-query` | Async State/Caching (optional) | ^5.x |

> **Warum Tailwind v4?** Tailwind v4 ist da und bietet native CSS-first Konfiguration,  
> `@tailwindcss/vite` Plugin, und `@theme`-Direktive statt tailwind.config.js.  
> Passt perfekt zu unserem CSS-Token-System aus test.html.  
> **Alternative:** Tailwind v3 + tailwind.config.js (robuster, mehr Plugins).  
> → Entscheidung nach Diskussion.

### 1.3 Anbindung ans Backend

```
React Component
    → useCards().search(query)
        → services/api.ts → invoke('search_cards', { args: { query } })
            → Tauri IPC Bridge
                → Rust commands.rs → SQLite / Scryfall
            → ← SearchResult
        → ← CardResponse[]
    → ← Rendern der Karten
```

Alle API-Aufrufe sind bereits in `FRONTEND_INTEGRATION.md` dokumentiert.  
Die `api.ts` wird mit den **aktuellen Typen aus commands.rs** (15+ statt 8 Commands) neu geschrieben.

---

## 2. Design System — Die Magic-Ästhetik

### 2.1 Design-Tokens aus test.html

Die `test.html` enthält ein vollständiges, durchdachtes Design-System.  
Alle Tokens werden in Tailwind v4 `@theme`-Direktive überführt:

```css
/* app.css — Tailwind v4 Design Tokens */
@import "tailwindcss";

@theme {
    /* Grundfarben */
    --color-base: #262425;
    --color-surface: #161515;
    --color-panel: #1f1d1e;
    --color-interactive: #2d2a2b;

    /* Akzentfarben */
    --color-parchment: #d9b88f;
    --color-leather: #bf8654;
    --color-wood: #8c583a;
    --color-crimson: #733030;

    /* Text */
    --color-text-main: #f5f2ed;
    --color-text-muted: #8c837e;

    /* Fonts */
    --font-serif: 'Cormorant Garamond', serif;
    --font-sans: 'Inter', sans-serif;
    --font-mono: 'JetBrains Mono', monospace;

    /* Radius */
    --radius-subtle: 6px;

    /* Motion — Spring-Kurven */
    --ease-spring-smooth: cubic-bezier(0.16, 1, 0.3, 1);
    --ease-spring-fast: cubic-bezier(0.25, 1, 0.5, 1);
}
```

### 2.2 Glas-Effekt (Glassmorphism)

Das Kern-Design-Pattern aus test.html:

```css
.glass-panel {
    background: rgba(22, 21, 21, 0.65);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(140, 88, 58, 0.16);
    border-radius: 6px;
}
```

→ Wird als `GlassPanel`-Komponente mit Tailwind `backdrop-blur` und `bg-black/65` umgesetzt.

### 2.3 Animationen

| Animations-Typ | Einsatz | Umsetzung |
|----------------|---------|-----------|
| Spring-Smooth | Page Transitions, Sidebar | `cubic-bezier(0.16, 1, 0.3, 1)` |
| Spring-Fast | Hover, Micro-Interaktionen | `cubic-bezier(0.25, 1, 0.5, 1)` |
| 3D Card Tilt | Karten-Preview | Framer Motion `useMotionValue` + `transform` |
| Holo Levitation | Hover-Effekte auf Cards | Translate Y + Shadow Scale |
| Fan-Out Deck | Deck-Vorschau | CSS transform + stagger |
| Card Flip | Karten-Rückseite | rotateY 180°, preserve-3d |

---

## 3. Navigationskonzept — "Die Planarpforte" (The Planar Gate)

**Keine Top-Navbar.** Stattdessen eine **Glass-Sidebar** auf der linken Seite:

### 3.1 Beschreibung

```
┌──────┬──────────────────────────────────────┐
│      │                                      │
│  ⚡  │                                      │
│      │                                      │
│  ≡   │           CONTENT AREA               │
│  🃏  │         (Page Content)               │
│  ⚔️  │                                      │
│  📖  │                                      │
│  ⚙️  │                                      │
│      │                                      │
│      │    Floating Mini-Nav (optional)      │
│      │         [ 🏠 ] [ ⬆ ]               │
└──────┴──────────────────────────────────────┘
```

- **Breite:** 64px im Ruhezustand (nur Icons)
- **Hover:** Erweitert sich auf ~200px mit sanfter Feder-Animation
- **Design:** Glassmorphism (halbtransparent, blur) — schwebt über dem Content
- **Icons:** Lucide + Magic-Flair (Mana-Symbole für Kategorien)
- **Aktiver Zustand:** Leuchtender Akzent (parchment/crimson) + animierte Anzeige

### 3.2 Sidebar-Items

| Icon | Label | Route | Beschreibung |
|------|-------|-------|-------------|
| 🏠 | Planar Hub | `/` | Übersicht / Dashboard |
| 🃏 | Sammlung | `/collection` | Kartensammlung + Suche |
| ⚔️ | Deck-Labor | `/deckbuilder` | Deckbau + Analyse |
| 📖 | Lore-Atlas | `/lore` | Story + Artbook |
| (—) | (Trenner) | — | — |
| ⚙️ | Einstellungen | `/settings` | App-Konfiguration |

### 3.3 Page-Transitions

Beim Seitenwechsel:
1. Aktuelle Seite fade-out + scale(0.98)
2. Kurze Pause (80ms)
3. Neue Seite fade-in + scale(1) mit Spring-Kurve

→ Umsetzung mit `<AnimatePresence>` + `<motion.div>` um den Router-Outlet.

### 3.4 Alternative: Hub-Startseite

Optional: Die App startet auf einer **Hub-Seite** mit drei großen, kartenähnlichen Eintritts-Tiles:

```
┌──────────────────────────────────────────┐
│                                          │
│   ┌───────────┐ ┌───────────┐ ┌───────┐ │
│   │  🃏       │ │  ⚔️       │ │ 📖   │ │
│   │ Sammlung  │ │ Deck-Labor│ │Lore   │ │
│   │           │ │           │ │Atlas  │ │
│   └───────────┘ └───────────┘ └───────┘ │
│                                          │
└──────────────────────────────────────────┘
```

Dieses Konzept kann **parallel zur Sidebar** existieren:  
Sidebar = schnelle Navigation von überall, Hub = immersiver Einstieg.

---

## 4. Routing-Struktur

```tsx
<Routes>
  <Route element={<AppShell />}>
    <Route index element={<HubPage />} />
    <Route path="collection" element={<CollectionPage />} />
    <Route path="deckbuilder" element={<DeckbuilderPage />} />
    <Route path="deck/:id" element={<DeckDetailPage />} />
    <Route path="lore" element={<LoreAtlasPage />} />
    <Route path="lore/:id" element={<LoreDetailPage />} />
    <Route path="settings" element={<SettingsPage />} />
  </Route>
</Routes>
```

`AppShell` enthält die Sidebar und den `<AnimatedOutlet>` mit Page-Transitions.

---

## 5. State Management (Zustand)

```typescript
// store/appStore.ts — Globaler App-Status
interface AppState {
    sidebarExpanded: boolean;
    activePage: string;
    // Aktionen
    toggleSidebar: () => void;
    setActivePage: (page: string) => void;
}

// store/collectionStore.ts — Sammlungs-Status
interface CollectionState {
    cards: CardResponse[];
    totalCount: number;
    searchQuery: string;
    isLoading: boolean;
    // Aktionen
    searchCards: (query: string) => Promise<void>;
    clearSearch: () => void;
}

// store/deckStore.ts — Deck-Editor-Status
interface DeckState {
    decks: DeckResponse[];
    activeDeck: DeckResponse | null;
    // Aktionen
    loadDecks: () => Promise<void>;
    setActiveDeck: (deck: DeckResponse | null) => void;
}
```

Jeder Store wrappt die Tauri `invoke()`-Aufrufe.  
Für komplexeres Caching kann später `@tanstack/react-query` ergänzt werden.

---

## 6. Umsetzungs-Phasen

### 🔧 Phase 1 — Setup (einmalig)
- [ ] `npm install` + Dependencies ergänzen
- [ ] Tailwind v4 + PostCSS + `@tailwindcss/vite` einrichten
- [ ] Google Fonts (Cormorant Garamond, Inter, JetBrains Mono) einbinden
- [ ] `index.css` mit Tailwind + Design-Tokens befüllen
- [ ] TypeScript-Typen in `src/types/` definieren
- [ ] `src/services/api.ts` mit allen 15+ Tauri-Commands schreiben

### 🏗️ Phase 2 — Layout & Navigation
- [ ] `AppShell` mit Sidebar + Content-Area
- [ ] Sidebar-Komponente (Glassmorphism, Hover-Expand, Icons)
- [ ] `AnimatedOutlet` mit Framer Motion Page Transitions
- [ ] Router-Struktur + Lazy Loading

### 🎨 Phase 3 — Design System Components
- [ ] `GlassPanel` — Die universelle Container-Komponente
- [ ] `Button` — Primär, Sekundär, Ghost (Crimson/Parchment/Leather)
- [ ] `CardPreview` — MTG-Kartenanzeige (Bild, Name, Mana, Typ)
- [ ] `SearchInput` — Glassmorphism-Suche mit Icon
- [ ] `LoadingSpinner` / `SkeletonLoader`
- [ ] `Modal` — Overlay-Dialog
- [ ] `PageHeader` — Seitentitel + Aktionen

### 📄 Phase 4 — Pages mit Inhalt
- [ ] **CollectionPage:** Suchleiste, CardGrid, Filter, Stats
- [ ] **DeckbuilderPage:** Deck-Liste, Erstellen-Button
- [ ] **DeckDetailPage:** Drag&Drop-Karten, Mana-Kurve, Analyse
- [ ] **LoreAtlasPage:** Lore-Einträge, Timeline, Reader
- [ ] **HubPage:** 3 große Eintrittskarten + Willkommen

### ✨ Phase 5 — Animationen & Feinschliff
- [ ] 3D Card Tilt auf CardPreview
- [ ] Hover-Effekte (Levitation, Glare)
- [ ] Page-Transition-Animationen
- [ ] Sidebar-Hover-Animation (icons → text expand)
- [ ] Loading-Skeletons
- [ ] Micro-Interaktionen (Buttons, Links)

---

## 7. Wichtige Design-Entscheidungen zur Diskussion

### 7.1 Tailwind v3 vs v4

**Tailwind v4** (bald stabil): CSS-first config, `@theme`, `@tailwindcss/vite`,  
kein `tailwind.config.js` mehr. Zukunftsorientiert, aber relativ neu.

**Tailwind v3** (aktuell stabil): `tailwind.config.js`, max Kontrolle,  
riesiges Plugin-Ökosystem, alle wissen wie's geht.

→ **Empfehlung:** Tailwind v3 für Stabilität + Zukunftssicherheit durch klare Trennung.  
   Wir können später ohne großen Aufwand auf v4 migrieren.

### 7.2 Framer Motion vs CSS-Animationen

Framer Motion für:
- Page Transitions (enter/exit mit Keyframes)
- 3D Card Tilt (useMotionValue, useSpring)
- Layout-Animationen (AnimatePresence)

CSS für:
- Hover (Card Levitation, Fan-Out) — reine CSS-Selektoren, kein JS-Overhead
- Sidebar Hover-Expand
- Glassmorphism

### 7.3 Navigation: Sidebar vs Alternative

**Sidebar (primäre Idee):**
- Vorteil: Immer erreichbar, kein "top nav" Gefühl, passt zum Magic-Flair
- Nachteil: Nimmt Platz weg (aber nur 64px)

**Alternative: Command Palette (⌘K):**
- Spotlight-ähnliche Suche zum Navigieren
- Kann zusätzlich zur Sidebar existieren

### 7.4 Hub-Seite als Startseite

Soll die App auf einer **Dashboard/Hub-Seite** starten  
mit großen Card-Tiles zu den drei Hauptbereichen?  
Oder direkt in die letzte aktive Seite springen?

---

## 8. UI-Komponenten-Spezifikation

### GlassPanel (Allgegenwärtig)
```tsx
<GlassPanel className="p-6">
    // Inhalt
</GlassPanel>
```
- Hintergrund: `bg-black/65 backdrop-blur-xl`
- Border: `border border-wood/15`
- Radius: `rounded-subtle`

### CardPreview (MTG Card)
```tsx
<CardPreview 
    card={cardData} 
    variant="compact" // | "full" | "grid"
    tiltable={true}   // 3D Parallax-Tilt
/>
```
- Zeigt: Bild, Name, Mana-Kosten, Typzeile
- Hover: Levitation + Glare + Schatten-Anpassung
- Click: Öffnet Detail-Modal

### SearchInput (Collection, Deck Suche)
```tsx
<SearchInput
    placeholder="Karte suchen..."
    value={query}
    onChange={setQuery}
    glass // Glassmorphism-Stil
/>
```
- Glassmorphism-Hintergrund
- Such-Icon links
- Clear-Button rechts bei Eingabe
- Debounced (300ms)

---

## 9. Nächste Schritte

1. **Plan besprechen** — Feedback einholen, Entscheidungen treffen
2. **Phase 1 Setup umsetzen** — Dependencies, Config, API-Client
3. **Phase 2 Layout** — Sidebar + AppShell + Router
4. **Phase 3 Components** — UI-Primitives bauen
5. **Phase 4 Pages** — Inhalt + API-Anbindung
6. **Phase 5 Animationen** — Feinschliff

Nach jeder Phase: Review & Anpassung.

---

*Dieses Dokument ist ein lebender Plan. Er wird während der Umsetzung iterativ verfeinert.*
