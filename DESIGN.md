---
version: alpha
name: MTG Multiverse Studio
description: Dark-fantasy MTG companion app — architectural minimalism meets TCG gravitas. A desktop studio for collection management, deck building, lore exploration, and local-AI-assisted play.
colors:
  primary: "#d9b88f"
  bg-base: "#262425"
  bg-surface: "#161515"
  bg-panel: "#1f1d1e"
  bg-interactive: "#2d2a2b"
  bg-glass: "rgba(22, 21, 21, 0.65)"
  parchment: "#d9b88f"
  leather: "#bf8654"
  wood: "#8c583a"
  gold: "#c9a84c"
  crimson: "#733030"
  leaf: "#2d5a27"
  storm: "#4a5f7a"
  text-main: "#f5f2ed"
  text-muted: "#8c837e"
  text-dim: "#5a5040"
  glass-border: "rgba(140, 88, 58, 0.16)"
  glass-border-strong: "rgba(255, 255, 255, 0.12)"
typography:
  h1:
    fontFamily: Cormorant Garamond
    fontSize: 38px
    fontWeight: 700
    lineHeight: 1.15
  h2:
    fontFamily: Cormorant Garamond
    fontSize: 28px
    fontWeight: 600
    lineHeight: 1.25
  h3:
    fontFamily: Cormorant Garamond
    fontSize: 20px
    fontWeight: 600
    lineHeight: 1.3
  body-md:
    fontFamily: Inter
    fontSize: 1rem
    fontWeight: 400
    lineHeight: 1.6
  body-sm:
    fontFamily: Inter
    fontSize: 0.875rem
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: 500
    letterSpacing: "0.12em"
    textTransform: uppercase
  caption:
    fontFamily: JetBrains Mono
    fontSize: 10px
    fontWeight: 400
  mtg-name:
    fontFamily: Cormorant Garamond
    fontSize: 12px
    fontWeight: 700
    lineHeight: 1.2
  mtg-rules:
    fontFamily: Inter
    fontSize: 10px
    fontWeight: 400
    lineHeight: 1.35
rounded:
  subtle: 6px
  card: 10px
  panel: 12px
  pill: 9999px
spacing:
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
  xxl: 48px
components:
  glass-panel-default:
    backgroundColor: "{colors.bg-glass}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.subtle}"
  glass-panel-strong:
    backgroundColor: "{colors.bg-glass}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.subtle}"
  button-primary:
    backgroundColor: "{colors.parchment}"
    textColor: "{colors.bg-base}"
    rounded: "{rounded.subtle}"
    padding: "8px 16px"
  button-primary-hover:
    backgroundColor: "{colors.leather}"
    textColor: "{colors.bg-base}"
  button-secondary:
    backgroundColor: "{colors.bg-interactive}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.subtle}"
    padding: "8px 16px"
  button-secondary-hover:
    backgroundColor: "{colors.wood}"
    textColor: "{colors.text-main}"
  button-ghost:
    backgroundColor: transparent
    textColor: "{colors.text-main}"
    rounded: "{rounded.subtle}"
    padding: "8px 16px"
  button-ghost-hover:
    backgroundColor: "{colors.bg-interactive}"
  button-danger:
    backgroundColor: "{colors.crimson}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.subtle}"
    padding: "8px 16px"
  button-glass:
    backgroundColor: "{colors.bg-glass}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.subtle}"
    padding: "8px 16px"
  button-glass-hover:
    backgroundColor: "rgba(22, 21, 21, 0.8)"
  mtg-card-frame:
    backgroundColor: "#110f10"
    width: "260px"
    height: "364px"
    rounded: "{rounded.card}"
  mtg-card-inner:
    backgroundColor: "#110f10"
    rounded: "{rounded.card}"
    padding: "10px"
  nav-tab-default:
    backgroundColor: transparent
    textColor: "{colors.text-muted}"
    rounded: "{rounded.pill}"
    padding: "6px 16px"
  nav-tab-active:
    backgroundColor: "{colors.bg-interactive}"
    textColor: "{colors.parchment}"
  input-search:
    backgroundColor: "{colors.bg-surface}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.pill}"
    padding: "8px 16px"
  input-search-focus:
    backgroundColor: "{colors.bg-surface}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.pill}"
    padding: "8px 16px"
  kbd-badge:
    backgroundColor: "{colors.bg-interactive}"
    textColor: "{colors.text-main}"
    rounded: "{rounded.subtle}"
    padding: "2px 6px"
  loader-spinner:
    rounded: "50%"
---

## Overview

MTG Multiverse Studio ist eine Dark-Fantasy-Desktop-App für Magic: The Gathering — ein digitales Studio, das Sammlungsmanagement, Deckbau, Lore-Exploration und lokale KI-Assistenz in einer App vereint. Die visuelle Identität verbindet **architektonischen Minimalismus** (klare Raster, reduziertes Layout) mit **TCG-Gravitas** (Pergament-Töne, Ledertöne, rituelle Typographie). Jede Oberfläche fühlt sich an wie ein Schreibtisch in einer verwinkelten Bibliothek: dunkle Obsidian-Oberflächen, warme Akzente aus vergilbtem Pergament und abgegriffenem Leder, sanfte Glassmorphism-Ebenen.

Das Design kommuniziert Ehrfurcht vor dem Spiel, ohne nostalgisch zu wirken. Helle Elemente treten selten und gezielt auf — der Fokus bleibt auf den Magic Cards selbst. Die MTG-Karten-Vorschau-Komponente ist das Herzstück und erhält ein eigenes 3D-Transformationssystem (Hover-Lift, Glare-Effekte, 22px Art-Box-Tiefe im 3D-Raum).

## Colors

Die Palette ist um einen dunkel-mahagonifarbenen Hintergrund zentriert, mit einer warmen Pergament-Leder-Akzentfamilie und einem einzigen dezenten Signalrot (`crimson`) für destruktive Aktionen.

### Backgrounds

- **bg-base (#262425):** Seiten-Hintergrund. Ein tiefes, warmes Dunkelbraun — kein reines Schwarz. Gibt dem Auge eine weiche Basis.
- **bg-surface (#161515):** Karten/Surface-Träger. Noch dunkler, fast obsidianfarben. Für SearchInputs, Badges, dicht gepackte Listen.
- **bg-panel (#1f1d1e):** Panel-Container. Mittelton zwischen base und surface.
- **bg-interactive (#2d2a2b):** Buttons (secondary), Tab-aktive, interaktive Hover-Zustände.

### Accents (Pergament-Familie)

- **primary / parchment (#d9b88f):** Primärer Akzent — der „Goldton" der App. Überschriften, Button-Primary, aktive Navigation, Links, Loader. Jeder Einsatz ist bewusst gesetzt.
- **leather (#bf8654):** Hover-Zustand von Parchment, Label-Farbe, sekundäre Textakzente.
- **wood (#8c583a):** Border-Linien, Hover von Secondary-Buttons, Scrollbar-Hover. Der strukturelle Akzent.
- **gold (#c9a84c):** Seltene Auszeichnungen (Mana-Kurve, Seltenheits-Rahmen). Wird extrem sparsam eingesetzt.

### Semantics

- **crimson (#733030):** Einzige semantische Farbe ausserhalb der Pergament-Familie. Reserviert für Löschen, Gefahr, Zerstörung.

### Text

- **text-main (#f5f2ed):** Warmes, leicht cremiges Weiss. Fliesstext, Labels, primäre Informationen.
- **text-muted (#8c837e):** Sekundärer Text, Metadaten, Platzhalter. Lesbar, aber zurückhaltend.
- **text-dim (#5a5040):** Die unterste Lesestufe. Für Fussnoten und deaktivierte Zustände.

### Glassmorphism

Alle `glass-panel`-Varianten teilen `backdrop-filter: blur(12px)` mit einer halbtransparenten `rgba(22, 21, 21, 0.65)`-Füllung. Die Border-Variation (`glass-border` vs `glass-border-strong`) steuert die Sichtbarkeit des Glaseffekts.

## Typography

Das Font-System verwendet bewusst drei Familien — eine serifenbetonte für Überschriften und Karten-Namen (Cormorant Garamond), eine serifenlose für Fliesstext (Inter) und eine Monospace für Codes, Labels und Metadaten (JetBrains Mono).

- **Cormorant Garamond** — Nur für Hierarchie: h1-h3 und MTG-Kartennamen. Kein Body-Text. Die Serifen verleihen dem UI eine bibliothekarische Würde.
- **Inter** — Body-Text, Buttons, Inputs. Funktionale Klarheit.
- **JetBrains Mono** — Labels (`text-transform: uppercase, letter-spacing: 0.12em`), Captions, Kürzel, Keyboard-Badges. Bringt Rhythmus in dichte Informationsanzeigen.

### Scale

| Token | Size | Weight | Line Height | Familie |
|-------|------|--------|-------------|---------|
| h1 | 38px | 700 | 1.15 | Cormorant |
| h2 | 28px | 600 | 1.25 | Cormorant |
| h3 | 20px | 600 | 1.30 | Cormorant |
| body-md | 16px | 400 | 1.60 | Inter |
| body-sm | 14px | 400 | 1.50 | Inter |
| label | 11px | 500 | — | JetBrains Mono (uppercase, 0.12em tracking) |
| caption | 10px | 400 | — | JetBrains Mono |

### MTK Card Typography

MTG-Karten verwenden ein separates, deutlich kleineres Set, um das Kartenlayout originalgetreu nachzubilden:

| Rolle | Size | Weight | Color |
|-------|------|--------|-------|
| Kartenname | 12px, Cormorant, 700 | Fett | #1a191a |
| Regelltext | 10px, Inter, 400 | Normal | #262425 |

## Layout & Spacing

Das Raster basiert auf einem **4px-Baseline-Grid**. Die maximale Inhaltsbreite beträgt 1120px (optional 1400px für Suchansichten).

| Ebene | Token | Wert | Verwendung |
|-------|-------|------|------------|
| Mikro | xs | 4px | Icon zu Text in Buttons |
| Eng | sm | 8px | Intra-Komponente (Button-Icon-zu-Label) |
| Standard | md | 16px | Zwischen Komponenten (Card-Gap, Form-Feld-Gap) |
| Gross | lg | 24px | Section-Gap, Card-Padding |
| Extra | xl | 32px | Zwischen Feature-Blöcken |
| Sektion | xxl | 48px | Seiten-Sections voneinander trennen |

Der Body-Seiten-Padding beträgt standardmässig 16px (mobil) bis 32px (desktop). Der Header ist `sticky` mit `top: 0` und `backdrop-filter: blur(12px)`.

## Elevation & Depth

Drei Schatten-Ebenen:

1. **shadow-card** — Karten, Glass-Panels, Dropdowns (`0 10px 25px rgba(0,0,0,0.5)`)
2. **shadow-card-hover** — Gehoverte Karten, Modal-Vordergrund (`0 25px 55px rgba(0,0,0,0.8)`)
3. **shadow-glass** — Floatende Glass-Elemente (`0 15px 35px rgba(0,0,0,0.4)`)

**3D-Karten-Tiefe:** Die MTG-Karten-Vorschau nutzt einen 3D-Raum via `perspective: 1200px`. Bei Hover schwebt die Karte 22px nach oben (`translateY(-22px) scale(1.03)`) und die Art-Box erhält einen `translateZ(22px)`-Pop-out-Effekt mit einem radialen Glare-Overlay.

## Shapes

- **subtle (6px):** Standard-Radius — Buttons, Inputs, Glass-Panels, Badges. Die meistgenutzte Rundung.
- **card (10px):** MTG-Kartenrahmen, Ergebnis-Karten im Raster.
- **panel (12px):** (vorgesehen für grössere Modal-Container)
- **pill (9999px):** Nur für Nav-Tabs und SearchInput.

## Components

### glass-panel / glass-panel-strong
Das primäre Container-Element. `glass-panel` für Karten und Sidebars, `glass-panel-strong` für hervorgehobene Bereiche (Hero, Aktionszentren).

### button-{primary,secondary,ghost,danger,glass}
Fünf Button-Varianten. Jede hat einen `-hover`-Gegenpart als separate Component (keine Verschachtelung). Der Button-Text ist immer `Inter 500`, 16px. Icons sind linksbündig (default) oder rechtsbündig via `iconPosition`.

- **primary:** Pergament-Button auf dunklem Grund — die einzige High-Emphasis-Aktion pro Screen.
- **secondary:** Umrandeter Button mit interaktivem Hintergrund.
- **ghost:** Rein textbasiert, ohne Füllung.
- **danger:** Crimson-Hintergrund für Delete/Zerstören.
- **glass:** Halbtransparenter Button mit Glas-Effekt.

### mtg-card-frame / mtg-card-inner
Die MTG-Karten-Vorschau ist das komplexeste Component. Sie besteht aus einem äusseren Frame (260×364px) und einem Inneren mit:

1. Mana-Reihe (parchment-gradient)
2. Art-Box (Kartenbild mit 3D-Transform)
3. Text-Box (heller Fond #dfd7cb für Lesbarkeit)
4. Glare-Effekt (weißer radialer Gradient, opak bei Hover)
5. Foil-Variante mit Shimmer-Animation

### nav-tab
Navigation-Tabs für den Tab-Balken. Aktiv: `bg-interactive + parchment-text`. Default: transparent + muted-text. `uppercase`, `font-size: 0.75rem`, `tracking: 0.08em`.

### input-search
Runder Such-Input mit Pill-Radius. Default-Zustand mit wood-border, Focus mit parchment-border + schwachem Glow.

### kbd-badge
Keyboard-Shortcut-Badge. `bg-interactive` + `text-main` erfüllt WCAG AA (Contrast > 4.5:1). `font-family: JetBrains Mono`, `font-size: 10px`.

### deck-stack
Fan-Out-Effekt für Deck-Vorschauen. Bis zu 5 Karten werden mit `perspective: 1200px` gestapelt, jede mit 40ms Delay-Offset für kaskadierte Animationen.

### loader-spinner
Runder Lade-Indikator mit `border-color` = parchment, `border-top-color` = transparent, animiert via `@keyframes spin`.

## Motion

Zwei Easing-Curves steuern alle Animationen:

| Curve | Einsatz | Dauer |
|-------|---------|-------|
| `cubic-bezier(0.16, 1, 0.3, 1)` | Page-Transitions, Card-Hover-Lift, List-Enter | 500ms (smooth) |
| `cubic-bezier(0.25, 1, 0.5, 1)` | Button-Hover, Tab-Wechsel, Input-Focus | 220ms (fast) |

### Schlüssel-Animationen

1. **Card Hover Lift:** 250ms, Karte schwebt 22px hoch + 3% Skalierung + Box-Shadow-Wechsel zu hover.
2. **Page Transition:** 500ms smooth — fade + scale(0.98→1) beim Enter, reverse beim Exit.
3. **Fan-Out Delay:** Jede Karte im Deck-Stapel hat 40ms mehr Delay (1: 0ms, 2: 40ms, 3: 80ms…).
4. **Glare Reveal:** 200ms ease — radiales Overlay wird bei Hover sichtbar.
5. **Spinner:** `@keyframes spin` auf dem Loader-Ring.

## Do's and Don'ts

- **Do** use token references (`{colors.parchment}`) in component definitions instead of hardcoding hex values.
- **Do** extend the palette via frontmatter before introducing an ad-hoc color.
- **Don't** nest component variants — `button-primary-hover` is a sibling key, not a child of `button-primary`.
- **Don't** use the `h1` font-size (38px) for anything other than page-level headings.
- **Don't** add more than one `button-primary` per screen — it dilutes the emphasis signal.
- **Do** apply `backdrop-filter` only through the glass-panel utility classes to keep blur consistent.
- **Don't** use `crimson` for anything other than destructive actions.
- **Do** wrap card hover effects in `transform-gpu` for performance.
- **Don't** introduce font families outside the three canonical ones without updating the typography section.
