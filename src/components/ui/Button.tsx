import { cn } from '../../utils/cn';
import { LucideIcon } from 'lucide-react';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'glass';
  size?: 'sm' | 'md' | 'lg';
  icon?: LucideIcon;
  iconPosition?: 'left' | 'right';
  loading?: boolean;
  fullWidth?: boolean;
}

export function Button({
  children,
  className,
  variant = 'primary',
  size = 'md',
  icon,
  iconPosition = 'left',
  loading = false,
  fullWidth = false,
  disabled,
  ...props
}: ButtonProps) {
  const baseStyles = 'inline-flex items-center justify-center font-medium transition-fast focus-ring disabled:opacity-50 disabled:cursor-not-allowed';

  const variantStyles = {
    primary: 'bg-[var(--color-parchment)] text-[var(--color-bg-base)] hover:bg-[var(--color-leather)] active:bg-[var(--color-wood)]',
    secondary: 'bg-[var(--color-bg-interactive)] text-[var(--color-text-main)] hover:bg-[var(--color-wood)] border border-[var(--color-wood)]',
    ghost: 'bg-transparent text-[var(--color-text-main)] hover:bg-[var(--color-bg-interactive)]',
    danger: 'bg-[var(--color-crimson)] text-[var(--color-text-main)] hover:bg-[#8b3a3a]',
    glass: 'glass-panel text-[var(--color-text-main)] hover:bg-[rgba(22,21,21,0.8)]',
  };

  const sizeStyles = {
    sm: 'px-3 py-1.5 text-sm gap-1.5',
    md: 'px-4 py-2 text-base gap-2',
    lg: 'px-6 py-3 text-lg gap-2.5',
  };

  const widthStyle = fullWidth ? 'w-full' : '';

  return (
    <button
      className={cn(baseStyles, variantStyles[variant], sizeStyles[size], widthStyle, className)}
      disabled={disabled || loading}
      {...props}
    >
      {loading && (
        <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="3" fill="none" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
      )}
      {icon && iconPosition === 'left' && !loading && <span className="flex-shrink-0">{icon}</span>}
      <span>{children}</span>
      {icon && iconPosition === 'right' && !loading && <span className="flex-shrink-0">{icon}</span>}
    </button>
  );
}