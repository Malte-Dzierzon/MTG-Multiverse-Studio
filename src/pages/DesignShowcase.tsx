import React, { useState } from 'react';
import { motion, useScroll, useTransform } from 'framer-motion';
import PixelBlast from '../components/effects/PixelBlast';

// ─── Exact Colors from test.html ──────────────────────────────────────
const COLORS = {
  bgBase: '#262425',
  bgSurface: '#161515',
  bgPanel: '#1f1d1e',
  bgInteractive: '#2d2a2b',
  parchment: '#d9b88f',
  leather: '#bf8654',
  wood: '#8c583a',
  crimson: '#733030',
  textMain: '#f5f2ed',
  textMuted: '#8c837e',
} as const;

// ─── Color Cards Data ─────────────────────────────────────────────────
const colorCards = [
  { name: 'Base Dunkelbraun', hex: COLORS.bgBase, role: 'Seiten-Hintergrund', text: COLORS.textMain },
  { name: 'Surface Obsidian', hex: COLORS.bgSurface, role: 'Karten-Träger', text: COLORS.textMain },
  { name: 'Panel Container', hex: COLORS.bgPanel, role: 'Panel-Fläche', text: COLORS.textMain },
  { name: 'Interactive Control', hex: COLORS.bgInteractive, role: 'Buttons & Hover', text: COLORS.textMain },
  { name: 'Parchment Light', hex: COLORS.parchment, role: 'Primärer Akzent', text: '#1a191a' },
  { name: 'Leather Accent', hex: COLORS.leather, role: 'Sekundärer Akzent', text: '#1a191a' },
  { name: 'Wood Framer', hex: COLORS.wood, role: 'Border-Linien', text: COLORS.textMain },
  { name: 'Crimson Core', hex: COLORS.crimson, role: 'Destruktive Aktionen', text: COLORS.textMain },
];

// ─── Feature Cards ────────────────────────────────────────────────────
const features = [
  { icon: '🎨', title: 'Farbpalette', desc: '8 abgestimmte Farben von Obsidian bis Pergament – jede mit definierter Rolle im UI.' },
  { icon: '🔤', title: 'Typografie', desc: 'Cormorant Garamond für Überschriften, Inter für Fliesstext, JetBrains Mono für Code.' },
  { icon: '🪟', title: 'Glassmorphism', desc: 'Halbdurchsichtige Panels mit backdrop-filter: blur(12px) und konsistentem 6px Radius.' },
  { icon: '🃏', title: 'MTG Card Engine', desc: '3D-Kartenrahmen mit 260×364px, Hover-Lift, Glare-Effekt und holografischem Schimmer.' },
  { icon: '✨', title: 'Motion Design', desc: 'Zwei Easing-Curves (smooth/fast) steuern alle Übergänge – von Page-Transitions bis Card-Hover.' },
  { icon: '📐', title: '4px Baseline Grid', desc: 'Einheitliches Spacing von 4px (Mikro) bis 48px (Sektionen) für konsistente Proportionen.' },
];

// ─── MTG Card Component ──────────────────────────────────────────────
function MtgCard({ hover = 'levitate' }: { hover?: 'levitate' | 'stack' | 'none' }) {
  const [flipped, setFlipped] = useState(false);

  if (hover === 'stack') {
    return (
      <div className="relative w-[200px] h-[280px] mx-auto" style={{ perspective: '1200px' }}>
        {[0, 1, 2].map((i) => (
          <motion.div
            key={i}
            className="absolute inset-0 rounded-[10px] overflow-hidden cursor-pointer"
            style={{
              background: '#110f10',
              border: '1px solid ' + COLORS.wood,
              zIndex: 3 - i,
            }}
            initial={false}
            whileHover={{
              rotate: i === 0 ? -12 : i === 1 ? -2 : 10,
              x: i === 0 ? -80 : i === 1 ? 0 : 80,
              y: i === 0 ? -10 : i === 1 ? -20 : -8,
              scale: 1.02,
              transition: { type: 'spring', stiffness: 200, damping: 20 },
            }}
          >
            <CardFace name={['Emerald Guardian', 'Blightsteel Colossus', 'Aetherflux Reservoir'][i]} />
          </motion.div>
        ))}
      </div>
    );
  }

  return (
    <div
      className="relative mx-auto cursor-pointer"
      style={{ perspective: '1200px', width: '220px', height: '308px' }}
      onClick={() => setFlipped(!flipped)}
    >
      <motion.div
        className="relative w-full h-full rounded-[10px] overflow-hidden"
        style={{
          background: '#110f10',
          border: '1px solid ' + COLORS.wood,
          transformStyle: 'preserve-3d',
        }}
        animate={{ rotateY: flipped ? 180 : 0 }}
        transition={{ duration: 0.65, ease: [0.16, 1, 0.3, 1] }}
      >
        {/* Front */}
        <div className="absolute inset-0 backface-hidden">
          <CardFace name="Blightsteel Colossus" />
        </div>
        {/* Back */}
        <div
          className="absolute inset-0 backface-hidden flex items-center justify-center"
          style={{ transform: 'rotateY(180deg)', background: '#110f10', border: '2px solid ' + COLORS.wood }}
        >
          <div
            className="w-[calc(100%-16px)] h-[calc(100%-16px)] flex items-center justify-center rounded-[6px]"
            style={{
              border: '1px dashed ' + COLORS.leather,
              background: `radial-gradient(circle, ${COLORS.crimson} 20%, #161515 80%)`,
            }}
          >
            <span className="font-serif text-2xl" style={{ color: COLORS.parchment }}>MTG</span>
          </div>
        </div>
      </motion.div>
      {hover === 'levitate' && (
        <motion.div
          className="absolute w-[90%] h-[14px] bottom-[-12px] left-[5%] rounded-full pointer-events-none"
          style={{
            background: 'rgba(0,0,0,0.7)',
            filter: 'blur(12px)',
          }}
          whileHover={{
            scaleY: 0.4,
            y: 18,
            opacity: 0.35,
            filter: 'blur(16px)',
          }}
        />
      )}
    </div>
  );
}

function CardFace({ name }: { name: string }) {
  return (
    <div className="w-full h-full p-[6px] flex flex-col" style={{ background: 'linear-gradient(145deg, #262425, #1c1a1b)' }}>
      {/* Mana row */}
      <div
        className="flex items-center justify-between px-2 py-[2px] rounded-[12px] mb-1"
        style={{
          background: `linear-gradient(to right, ${COLORS.parchment}, #bf9f75)`,
          border: '1px solid ' + COLORS.wood,
          color: '#1a191a',
          fontSize: '11px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.3)',
        }}
      >
        <h4 className="font-serif font-bold text-[11px] truncate mr-2">{name}</h4>
        <span className="font-mono font-bold shrink-0">{'{2}{W}{W}'}</span>
      </div>
      {/* Art box */}
      <div
        className="flex-1 relative overflow-hidden flex items-center justify-center mb-1"
        style={{
          background: '#110f10',
          border: '1px solid rgba(255,255,255,0.05)',
          transform: 'translateZ(22px)',
        }}
      >
        <div
          className="absolute inset-0"
          style={{
            background: 'url(https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?auto=format&fit=crop&q=80&w=500) center/cover',
            filter: 'grayscale(20%) sepia(10%)',
            opacity: 0.8,
          }}
        />
        <span
          className="relative z-[2] font-mono text-[9px] px-2 py-[2px] rounded-[4px] text-white uppercase tracking-[1px]"
          style={{ background: COLORS.crimson }}
        >
          Artwork
        </span>
      </div>
      {/* Text box */}
      <div
        className="h-[35%] p-1.5 flex flex-col gap-1 text-[9px]"
        style={{ background: '#dfd7cb', border: '1px solid ' + COLORS.wood, color: '#262425' }}
      >
        <p className="leading-[1.3] font-medium font-sans">
          Flying, first strike. When Blightsteel Colossus enters the battlefield, target opponent loses half their life, rounded up.
        </p>
        <p
          className="italic mt-auto pt-1 text-[10px]"
          style={{ fontFamily: "'Cormorant Garamond', serif", borderTop: '1px solid #c5b9a8', color: '#574e46' }}
        >
          "The core pulses with dark energy."
        </p>
        {/* PT badge */}
        <div
          className="absolute bottom-1 right-2 px-1.5 py-[1px] rounded-[4px] font-mono font-bold text-[9px]"
          style={{
            background: COLORS.parchment,
            border: '1px solid ' + COLORS.wood,
            color: '#262425',
            boxShadow: '0 2px 5px rgba(0,0,0,0.4)',
          }}
        >
          11/11
        </div>
      </div>
    </div>
  );
}

// ─── Section Wrapper ──────────────────────────────────────────────────
function Section({ id, title, subtitle, children, className = '' }: {
  id?: string; title: string; subtitle?: string; children: React.ReactNode; className?: string;
}) {
  return (
    <section id={id} className={`relative z-10 py-20 md:py-28 ${className}`}>
      <div className="max-w-7xl mx-auto px-4 sm:px-8 lg:px-12">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.7, ease: [0.16, 1, 0.3, 1] }}
          className="mb-12 md:mb-16"
        >
          <span
            className="inline-block font-mono text-[11px] uppercase tracking-[2px] mb-3"
            style={{ color: COLORS.leather }}
          >
            {id}
          </span>
          <h2
            className="font-serif text-3xl md:text-4xl lg:text-5xl font-bold leading-tight"
            style={{ color: COLORS.parchment }}
          >
            {title}
          </h2>
          {subtitle && (
            <p className="mt-3 text-base md:text-lg max-w-2xl" style={{ color: COLORS.textMuted }}>
              {subtitle}
            </p>
          )}
        </motion.div>
        {children}
      </div>
    </section>
  );
}

// ─── Glass Panel ──────────────────────────────────────────────────────
function GlassPanel({ children, className = '', strong = false }: {
  children: React.ReactNode; className?: string; strong?: boolean;
}) {
  return (
    <div
      className={`rounded-[6px] p-6 md:p-8 ${className}`}
      style={{
        background: 'rgba(22, 21, 21, 0.65)',
        backdropFilter: 'blur(12px)',
        WebkitBackdropFilter: 'blur(12px)',
        border: '1px solid ' + (strong ? 'rgba(255,255,255,0.12)' : 'rgba(140,88,58,0.16)'),
      }}
    >
      {children}
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════════
// PAGE
// ═══════════════════════════════════════════════════════════════════════
export default function DesignShowcase() {
  const { scrollYProgress } = useScroll();
  const heroOpacity = useTransform(scrollYProgress, [0, 0.15], [1, 0]);
  const heroScale = useTransform(scrollYProgress, [0, 0.15], [1, 0.95]);

  return (
    <div className="relative min-h-screen" style={{ background: COLORS.bgBase, color: COLORS.textMain }}>
      {/* ── PixelBlast Background ── */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <PixelBlast
          variant="circle"
          pixelSize={5}
          color={COLORS.parchment}
          patternScale={2.5}
          patternDensity={1.1}
          pixelSizeJitter={0.3}
          enableRipples
          rippleSpeed={0.35}
          rippleThickness={0.1}
          rippleIntensityScale={1.2}
          liquid={false}
          speed={0.4}
          edgeFade={0.3}
          transparent
        />
      </div>

      {/* ── Subtle gradient overlay ── */}
      <div
        className="fixed inset-0 z-[1] pointer-events-none"
        style={{
          background: `radial-gradient(ellipse at 50% 0%, ${COLORS.bgBase}00 0%, ${COLORS.bgBase} 70%)`,
        }}
      />

      {/* ════════════════════════════════════════ HERO ══════════════════ */}
      <motion.section
        style={{ opacity: heroOpacity, scale: heroScale }}
        className="relative z-10 min-h-screen flex flex-col items-center justify-center text-center px-4"
      >
        <div className="max-w-4xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
          >
            <span
              className="inline-block font-mono text-[11px] uppercase tracking-[3px] mb-6 px-4 py-2 rounded-full"
              style={{
                color: COLORS.parchment,
                background: 'rgba(22,21,21,0.65)',
                backdropFilter: 'blur(12px)',
                border: '1px solid rgba(140,88,58,0.16)',
              }}
            >
              MTG Multiverse Studio — Design System
            </span>

            <h1
              className="font-serif text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-bold leading-[0.95] mb-6"
              style={{ color: COLORS.parchment }}
            >
              Master Design<br />
              <span style={{ color: COLORS.textMain }}>Specification</span>
            </h1>

            <p className="text-lg sm:text-xl max-w-2xl mx-auto mb-10" style={{ color: COLORS.textMuted }}>
              Ein einheitliches visuelles System für Sammlungsmanagement, Deckbau,
              Lore-Exploration und lokale KI-Assistenz — verwurzelt in der Ästhetik
              von Magic: The Gathering.
            </p>

            <div className="flex flex-wrap items-center justify-center gap-4">
              <a
                href="#colors"
                className="inline-flex items-center gap-2 px-6 py-3 rounded-[6px] font-medium text-sm transition-all duration-300"
                style={{
                  background: COLORS.parchment,
                  color: '#1a191a',
                }}
                onMouseEnter={(e) => (e.currentTarget.style.background = COLORS.leather)}
                onMouseLeave={(e) => (e.currentTarget.style.background = COLORS.parchment)}
              >
                Palette erkunden
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" /></svg>
              </a>
              <a
                href="#cards"
                className="inline-flex items-center gap-2 px-6 py-3 rounded-[6px] font-medium text-sm transition-all duration-300"
                style={{
                  background: COLORS.bgInteractive,
                  color: COLORS.textMain,
                  border: '1px solid ' + COLORS.wood,
                }}
                onMouseEnter={(e) => { e.currentTarget.style.background = COLORS.wood; }}
                onMouseLeave={(e) => { e.currentTarget.style.background = COLORS.bgInteractive; }}
              >
                Karten ansehen
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" /></svg>
              </a>
            </div>
          </motion.div>
        </div>

        {/* Scroll indicator */}
        <motion.div
          className="absolute bottom-10"
          animate={{ y: [0, 8, 0] }}
          transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
        >
          <svg className="w-6 h-6" style={{ color: COLORS.wood }} fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </motion.div>
      </motion.section>

      {/* ════════════════════════════════════════ COLORS ══════════════════ */}
      <Section id="colors" title="Farbpalette" subtitle="Acht aufeinander abgestimmte Farben, die das visuelle System tragen – von tiefem Obsidian bis warmem Pergament.">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 md:gap-5">
          {colorCards.map((c, i) => (
            <motion.div
              key={c.name}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.05, ease: [0.16, 1, 0.3, 1] }}
            >
              <GlassPanel className="group cursor-default transition-transform duration-300 hover:scale-[1.02]">
                <div className="flex flex-col gap-4">
                  {/* Color swatch */}
                  <div
                    className="w-full h-24 rounded-[6px] transition-shadow duration-300 group-hover:shadow-xl"
                    style={{ background: c.hex }}
                  />
                  {/* Info */}
                  <div>
                    <h3 className="font-serif text-lg font-semibold" style={{ color: COLORS.parchment }}>
                      {c.name}
                    </h3>
                    <p className="font-mono text-xs mt-1" style={{ color: COLORS.textMuted }}>
                      {c.hex}
                    </p>
                    <p className="text-sm mt-2 leading-snug" style={{ color: COLORS.textMuted }}>
                      {c.role}
                    </p>
                  </div>
                </div>
              </GlassPanel>
            </motion.div>
          ))}
        </div>
      </Section>

      {/* ════════════════════════════════════════ TYPOGRAPHY ══════════════ */}
      <Section
        id="typography"
        title="Typografie"
        subtitle="Drei Schriftfamilien mit klaren Aufgaben: Cormorant Garamond für Überschriften, Inter für Fliesstext, JetBrains Mono für Code und Metadaten."
        className="bg-[#1f1d1e]/40"
      >
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <GlassPanel>
            <span className="font-mono text-[10px] uppercase tracking-[2px]" style={{ color: COLORS.leather }}>Serif Display</span>
            <h2 className="font-serif text-3xl mt-2" style={{ color: COLORS.parchment }}>Ancient Legend Vault</h2>
            <p className="font-mono text-[10px] mt-4" style={{ color: COLORS.textMuted }}>Cormorant Garamond · 28px · 600 Weight</p>
          </GlassPanel>

          <GlassPanel>
            <span className="font-mono text-[10px] uppercase tracking-[2px]" style={{ color: COLORS.leather }}>Sans UI Main</span>
            <p className="text-base mt-2 leading-relaxed" style={{ color: COLORS.textMain }}>
              Flying. When this creature enters the battlefield, trigger cache reload. <span style={{ color: COLORS.textMuted }}>(Inter · 16px · 400 Weight)</span>
            </p>
          </GlassPanel>

          <GlassPanel>
            <span className="font-mono text-[10px] uppercase tracking-[2px]" style={{ color: COLORS.leather }}>Data Mono</span>
            <p className="font-mono text-sm mt-2" style={{ color: COLORS.textMuted }}>SYS_LOG_ENV: 0x7FFF9D2A</p>
            <p className="font-mono text-[10px] mt-2" style={{ color: COLORS.textMuted }}>JetBrains Mono · 11px</p>
          </GlassPanel>

          <GlassPanel>
            <span className="font-mono text-[10px] uppercase tracking-[2px]" style={{ color: COLORS.leather }}>Scale Übersicht</span>
            <div className="mt-3 space-y-2">
              {[
                { token: 'h1', size: '38px', weight: '700', font: 'Cormorant' },
                { token: 'h2', size: '28px', weight: '600', font: 'Cormorant' },
                { token: 'h3', size: '20px', weight: '600', font: 'Cormorant' },
                { token: 'body', size: '16px', weight: '400', font: 'Inter' },
                { token: 'label', size: '11px', weight: '500', font: 'JetBrains Mono' },
                { token: 'caption', size: '10px', weight: '400', font: 'JetBrains Mono' },
              ].map((t) => (
                <div key={t.token} className="flex items-center gap-4 py-1 border-b" style={{ borderColor: 'rgba(140,88,58,0.1)' }}>
                  <span className="font-mono text-[10px] w-16 shrink-0" style={{ color: COLORS.leather }}>{t.token}</span>
                  <span className="font-mono text-[10px] w-16 shrink-0" style={{ color: COLORS.textMuted }}>{t.size}</span>
                  <span className="font-mono text-[10px] w-16 shrink-0" style={{ color: COLORS.textMuted }}>{t.weight}</span>
                  <span className="font-mono text-[10px]" style={{ color: COLORS.textMuted }}>{t.font}</span>
                </div>
              ))}
            </div>
          </GlassPanel>
        </div>
      </Section>

      {/* ════════════════════════════════════════ FEATURES ════════════════ */}
      <Section id="features" title="Design Tokens" subtitle="Jeder visuelle Baustein ist als Token definiert – Farben, Radien, Abstände, Schatten und Bewegungsprofile.">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {features.map((f, i) => (
            <motion.div
              key={f.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.05, ease: [0.16, 1, 0.3, 1] }}
            >
              <GlassPanel className="h-full group transition-all duration-300 hover:scale-[1.02]">
                <span className="text-2xl block mb-4">{f.icon}</span>
                <h3 className="font-serif text-lg font-semibold mb-2" style={{ color: COLORS.parchment }}>{f.title}</h3>
                <p className="text-sm leading-relaxed" style={{ color: COLORS.textMuted }}>{f.desc}</p>
              </GlassPanel>
            </motion.div>
          ))}
        </div>
      </Section>

      {/* ════════════════════════════════════════ CARDS ═══════════════════ */}
      <Section
        id="cards"
        title="MTG Card Engine"
        subtitle="Die Karten-Vorschau ist das Herzstück: 3D-Transforms, Hover-Lift, Glare-Effekte, Foil-Shimmer und interaktive Flip-Animation."
        className="bg-[#1f1d1e]/40"
      >
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Levitate Card */}
          <GlassPanel>
            <span className="font-mono text-[10px] uppercase tracking-[2px] mb-4 block" style={{ color: COLORS.leather }}>
              Levitation + Hover Lift
            </span>
            <MtgCard hover="levitate" />
            <p className="text-xs mt-6 text-center" style={{ color: COLORS.textMuted }}>
              Hover über die Karte · 22px Lift + 3% Scale + Schatten-Übergang
            </p>
          </GlassPanel>

          {/* Stack / Fan-out */}
          <GlassPanel>
            <span className="font-mono text-[10px] uppercase tracking-[2px] mb-4 block" style={{ color: COLORS.leather }}>
              Stacked Deck · Fan-Out
            </span>
            <MtgCard hover="stack" />
            <p className="text-xs mt-6 text-center" style={{ color: COLORS.textMuted }}>
              Hover über den Stapel · Kaskadiertes Auffächern mit Feder-Animation
            </p>
          </GlassPanel>
        </div>

        {/* Flip Card */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="mt-8"
        >
          <GlassPanel>
            <span className="font-mono text-[10px] uppercase tracking-[2px] mb-4 block" style={{ color: COLORS.leather }}>
              3D Flip · Click to Flip
            </span>
            <div className="flex justify-center">
              <MtgCard hover="none" />
            </div>
            <p className="text-xs mt-6 text-center" style={{ color: COLORS.textMuted }}>
              Klick auf die Karte · 180° Y-Rotation mit Smooth-Cubic-Easing
            </p>
          </GlassPanel>
        </motion.div>
      </Section>

      {/* ════════════════════════════════════════ MOTION ══════════════════ */}
      <Section id="motion" title="Motion & Easing" subtitle="Zwei kuratierte Cubic-Bezier-Kurven steuern das gesamte Bewegungssystem – von Mikro-Interaktionen bis Page-Transitions.">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <GlassPanel>
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-serif text-lg font-semibold" style={{ color: COLORS.parchment }}>Spring Smooth</h3>
              <span className="font-mono text-[10px] px-2 py-1 rounded-[4px]" style={{ background: COLORS.bgInteractive, color: COLORS.textMuted }}>500ms</span>
            </div>
            <div className="h-16 rounded-[6px] overflow-hidden relative" style={{ background: COLORS.bgSurface }}>
              <motion.div
                className="absolute top-2 left-2 w-12 h-12 rounded-[6px]"
                style={{ background: COLORS.parchment }}
                animate={{ x: [0, 320, 0] }}
                transition={{ duration: 2.5, repeat: Infinity, ease: [0.16, 1, 0.3, 1], repeatDelay: 1 }}
              />
            </div>
            <p className="font-mono text-[10px] mt-3" style={{ color: COLORS.textMuted }}>
              cubic-bezier(0.16, 1, 0.3, 1) · Page-Transitions, Card-Lift, List-Enter
            </p>
          </GlassPanel>

          <GlassPanel>
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-serif text-lg font-semibold" style={{ color: COLORS.parchment }}>Spring Fast</h3>
              <span className="font-mono text-[10px] px-2 py-1 rounded-[4px]" style={{ background: COLORS.bgInteractive, color: COLORS.textMuted }}>220ms</span>
            </div>
            <div className="h-16 rounded-[6px] overflow-hidden relative" style={{ background: COLORS.bgSurface }}>
              <motion.div
                className="absolute top-2 left-2 w-12 h-12 rounded-[6px]"
                style={{ background: COLORS.leather }}
                animate={{ x: [0, 320, 0] }}
                transition={{ duration: 1.5, repeat: Infinity, ease: [0.25, 1, 0.5, 1], repeatDelay: 0.5 }}
              />
            </div>
            <p className="font-mono text-[10px] mt-3" style={{ color: COLORS.textMuted }}>
              cubic-bezier(0.25, 1, 0.5, 1) · Button-Hover, Tab-Wechsel, Input-Focus
            </p>
          </GlassPanel>
        </div>
      </Section>

      {/* ════════════════════════════════════════ FOOTER ══════════════════ */}
      <footer className="relative z-10 border-t py-12 px-4" style={{ borderColor: 'rgba(140,88,58,0.15)' }}>
        <div className="max-w-7xl mx-auto text-center">
          <p className="font-serif text-lg" style={{ color: COLORS.parchment }}>MTG Multiverse Studio</p>
          <p className="font-mono text-xs mt-2" style={{ color: COLORS.textMuted }}>
            Design System · Version 1.0 · Fan Project under GPL-v3
          </p>
          <p className="font-mono text-[10px] mt-1" style={{ color: COLORS.wood }}>
            Daten von Scryfall · MTGJSON
          </p>
        </div>
      </footer>
    </div>
  );
}
