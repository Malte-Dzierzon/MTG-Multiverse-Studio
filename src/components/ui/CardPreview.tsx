import { cn } from '../../utils/cn';
import { formatManaCost } from '../../utils/helpers';
import type { CardResponse } from '../../types';

export interface CardPreviewProps {
  card: CardResponse;
  size?: 'small' | 'medium' | 'large';
  showQuantity?: number;
  showPrice?: boolean;
  showRarity?: boolean;
  variant?: 'default' | 'foil' | 'flip' | 'levitate' | 'deck-item';
  onClick?: () => void;
  className?: string;
}

const SIZE_CLASSES = {
  small: 'w-[160px] h-[224px]',
  medium: 'w-[200px] h-[280px]',
  large: 'w-[260px] h-[364px]',
};

const SIZE_TEXT_CLASSES = {
  small: 'text-xs',
  medium: 'text-sm',
  large: 'text-base',
};

export function CardPreview({
  card,
  size = 'medium',
  showQuantity,
  showPrice = false,
  showRarity = true,
  variant = 'default',
  onClick,
  className,
}: CardPreviewProps) {
  const sizeClass = SIZE_CLASSES[size];
  const textClass = SIZE_TEXT_CLASSES[size];
  
  const displayMana = formatManaCost(card.mana_cost);
  
  const rarityColor = {
    mythic: 'text-amber-400',
    rare: 'text-gold-300',
    uncommon: 'text-silver-300',
    common: 'text-stone-400',
    special: 'text-purple-400',
    bonus: 'text-pink-400',
  }[card.rarity?.toLowerCase()] || 'text-muted';

  const variantClasses = {
    default: 'mtg-card-frame',
    foil: 'mtg-card-frame mtg-foil-shimmer',
    flip: 'mtg-flip-stage',
    levitate: 'levitate-container',
    'deck-item': 'mtg-card-frame',
  };

  const renderCardFace = () => (
    <div className={cn('mtg-card-inner', variant === 'levitate' && 'levitate-card', variant === 'foil' && 'mtg-foil-shimmer')}>
      <div className="mtg-card-glare" />
      <div className="mtg-card-border">
        {/* Mana row */}
        <div className="mtg-mana-row">
          <h4 className="font-serif">{card.name}</h4>
          <span className="font-mono">{displayMana}</span>
        </div>

        {/* Art box */}
        <div className="mtg-art-box">
          {card.image_url_large ? (
            <img
              src={card.image_url_large}
              alt={card.name}
              className="w-full h-full object-cover"
              loading="lazy"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-muted">
              <span className="font-mono text-xs">Kein Bild</span>
            </div>
          )}
          {showRarity && (
            <div className="absolute top-2 right-2">
              <span className={cn('px-1.5 py-0.5 rounded font-mono text-xs border', rarityColor.replace('text-', 'border-'), `bg-[var(--color-bg-surface)]`)}>
                {card.rarity?.charAt(0).toUpperCase() + card.rarity?.slice(1)}
              </span>
            </div>
          )}
          {showQuantity && (
            <div className="absolute bottom-2 right-2">
              <span className={cn('px-2 py-0.5 rounded font-mono text-xs bg-[var(--color-parchment)] text-[var(--color-bg-base)] border border-[var(--color-wood)]')}>
                x{showQuantity}
              </span>
            </div>
          )}
        </div>

        {/* Text box */}
        <div className="mtg-text-box">
          <p className="mtg-rules-text">{card.card_text || '—'}</p>
          {card.type_line.includes('Creature') && card.power && card.toughness && (
            <div className="mtg-pt-badge font-mono">
              {card.power}/{card.toughness}
            </div>
          )}
        </div>
      </div>
    </div>
  );

  const renderFlipCard = () => (
    <div className="mtg-flip-card">
      <div className="mtg-card-face">
        {renderCardFace()}
      </div>
      <div className="mtg-card-face mtg-card-face-back">
        <div className="card-back-art">
          <div className="text-center">
            <div className="w-12 h-12 mx-auto mb-4 border-2 border-dashed border-[var(--color-leather)] rounded-full flex items-center justify-center">
              <span className="font-serif text-xl text-[var(--color-parchment)]">MTG</span>
            </div>
            <p className="font-mono text-xs text-[var(--color-text-muted)]">Rückseite</p>
          </div>
        </div>
      </div>
    </div>
  );

  const renderLevitateCard = () => (
    <>
      <div className="levitate-card">
        {renderCardFace()}
      </div>
      <div className="levitate-shadow" />
    </>
  );

  const cardContent = variant === 'flip' 
    ? renderFlipCard()
    : variant === 'levitate'
    ? renderLevitateCard()
    : renderCardFace();

  return (
    <div
      className={cn(
        variantClasses[variant],
        sizeClass,
        textClass,
        onClick && 'cursor-pointer',
        className
      )}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(); }} : undefined}
    >
      {cardContent}
      {showPrice && card.prices?.usd && (
        <div className="absolute bottom-2 left-2 right-2 text-right">
          <span className="font-mono text-xs text-[var(--color-parchment)] bg-[var(--color-bg-surface)]/80 px-2 py-0.5 rounded border border-[var(--color-wood)]">
            ${card.prices.usd}
          </span>
        </div>
      )}
    </div>
  );
}

// Stacked deck preview (fan-out effect)
export function DeckStackPreview({ 
  cards, 
  maxVisible = 5, 
  size = 'medium',
  className 
}: { 
  cards: CardResponse[]; 
  maxVisible?: number; 
  size?: 'small' | 'medium' | 'large';
  className?: string;
}) {
  const sizeClass = SIZE_CLASSES[size];
  
  const visibleCards = cards.slice(0, maxVisible);
  
  return (
    <div className={cn('deck-stack', sizeClass, className)}>
      {visibleCards.map((card, index) => (
        <div key={card.id} className="stacked-card fan-card" style={{ zIndex: maxVisible - index }}>
          <CardPreview card={card} size={size} variant="default" />
        </div>
      ))}
      {cards.length > maxVisible && (
        <div className="stacked-card fan-card absolute inset-0 flex items-center justify-center bg-[var(--color-bg-surface)]/90 backdrop-blur-sm">
          <span className="font-mono text-lg text-[var(--color-parchment)]">
            +{cards.length - maxVisible}
          </span>
        </div>
      )}
    </div>
  );
}