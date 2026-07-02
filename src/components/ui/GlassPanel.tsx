import { cn } from '../../utils/cn';

export interface GlassPanelProps {
  children: React.ReactNode;
  className?: string;
  variant?: 'default' | 'strong' | 'card';
  hover?: boolean;
  onClick?: () => void;
}

export function GlassPanel({ 
  children, 
  className, 
  variant = 'default',
  hover = false,
  onClick 
}: GlassPanelProps) {
  const baseClasses = {
    default: 'glass-panel',
    strong: 'glass-panel-strong',
    card: 'glass-card',
  };

  const hoverClasses = hover 
    ? (variant === 'card' ? 'glass-card-hover' : 'transition-smooth hover:shadow-[0_25px_55px_rgba(0,0,0,0.8)]')
    : '';

  const cursorClass = onClick ? 'cursor-pointer' : '';

  return (
    <div
      className={cn(baseClasses[variant], hoverClasses, cursorClass, className)}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(); }} : undefined}
    >
      {children}
    </div>
  );
}