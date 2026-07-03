import React from 'react';
import { cn } from '../../utils/cn';

export interface CardProps {
  children: React.ReactNode;
  className?: string;
  variant?: 'default' | 'elevated' | 'glass' | 'panel';
  hover?: boolean;
  padding?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
  onClick?: () => void;
}

export function Card({
  children,
  className,
  variant = 'default',
  hover = false,
  padding = 'md',
  onClick,
}: CardProps) {
  const variantStyles = {
    default: 'glass-card',
    elevated: 'glass-card',
    glass: 'glass-panel',
    panel: 'glass-panel-strong',
  };

  const hoverStyles = hover
    ? variant === 'elevated' || variant === 'default'
      ? 'glass-card-hover transition-smooth'
      : 'transition-smooth hover:shadow-[0_25px_55px_rgba(0,0,0,0.8)]'
    : '';

  const paddingStyles = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
    xl: 'p-10',
  };

  const cursorStyle = onClick ? 'cursor-pointer' : '';

  return (
    <div
      className={cn(variantStyles[variant], hoverStyles, paddingStyles[padding], cursorStyle, className)}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(); }} : undefined}
    >
      {children}
    </div>
  );
}

export interface CardHeaderProps {
  children: React.ReactNode;
  className?: string;
  action?: React.ReactNode;
}

export function CardHeader({ children, className, action }: CardHeaderProps) {
  return (
    <div className={cn('flex items-center justify-between mb-4', className)}>
      <div>{children}</div>
      {action && <div>{action}</div>}
    </div>
  );
}

export interface CardContentProps {
  children: React.ReactNode;
  className?: string;
}

export function CardContent({ children, className }: CardContentProps) {
  return <div className={cn('', className)}>{children}</div>;
}

export interface CardFooterProps {
  children: React.ReactNode;
  className?: string;
  justify?: 'start' | 'center' | 'end' | 'between';
}

export function CardFooter({ children, className, justify = 'end' }: CardFooterProps) {
  const justifyStyles = {
    start: 'justify-start',
    center: 'justify-center',
    end: 'justify-end',
    between: 'justify-between',
  };

  return (
    <div className={cn('flex items-center gap-3 mt-6 pt-4 border-t border-[rgba(140,88,58,0.1)]', justifyStyles[justify], className)}>
      {children}
    </div>
  );
}

export interface CardGridProps {
  children: React.ReactNode;
  className?: string;
  cols?: 1 | 2 | 3 | 4;
  gap?: 'sm' | 'md' | 'lg' | 'xl';
}

export function CardGrid({ children, className, cols = 3, gap = 'lg' }: CardGridProps) {
  const gridCols = {
    1: 'grid-cols-1',
    2: 'grid-cols-1 md:grid-cols-2',
    3: 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3',
    4: 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4',
  };

  const gapStyles = {
    sm: 'gap-4',
    md: 'gap-6',
    lg: 'gap-8',
    xl: 'gap-10',
  };

  return (
    <div
      className={cn(
        'grid',
        gridCols[cols],
        gapStyles[gap],
        className
      )}
    >
      {children}
    </div>
  );
}