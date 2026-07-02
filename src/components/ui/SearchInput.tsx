import { cn } from '../../utils/cn';
import { useState, useRef, useEffect, KeyboardEvent } from 'react';
import { Search, X, Command } from 'lucide-react';

export interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit?: (value: string) => void;
  placeholder?: string;
  className?: string;
  autoFocus?: boolean;
  showShortcut?: boolean;
}

export function SearchInput({
  value,
  onChange,
  onSubmit,
  placeholder = 'Karten suchen...',
  className,
  autoFocus = false,
  showShortcut = true,
}: SearchInputProps) {
  const [isFocused, setIsFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && onSubmit) {
      onSubmit(value);
    }
    if (e.key === 'Escape') {
      inputRef.current?.blur();
      onChange('');
    }
    // Cmd/Ctrl + K to focus
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      inputRef.current?.focus();
    }
  };

  return (
    <div className={cn('relative flex items-center', className)}>
      <div className={cn(
        'relative w-full transition-fast',
        isFocused && 'glass-panel-strong',
        !isFocused && 'glass-panel'
      )}>
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)] h-4 w-4 pointer-events-none transition-colors" />
        {showShortcut && (
          <kbd className="absolute left-8 top-1/2 -translate-y-1/2 hidden sm:block font-mono text-[10px] text-[var(--color-text-muted)] px-1.5 py-0.5 rounded bg-[var(--color-bg-surface)]/50 border border-[var(--color-wood)]/20">
            ⌘K
          </kbd>
        )}
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          placeholder={placeholder}
          className={cn(
            'w-full bg-transparent border-none outline-none text-[var(--color-text-main)] placeholder-[var(--color-text-muted)]',
            'font-sans text-sm',
            'pl-10 pr-10 py-2',
            showShortcut && 'pl-20 sm:pl-10',
            isFocused && 'placeholder-transparent'
          )}
        />
        {value && (
          <button
            type="button"
            onClick={() => onChange('')}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)] hover:text-[var(--color-parchment)] transition-colors p-1"
            aria-label="Suche löschen"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>
    </div>
  );
}

// Command Palette trigger
export function CommandPaletteTrigger({ onOpen }: { onOpen: () => void }) {
  return (
    <button
      onClick={onOpen}
      className="glass-panel px-3 py-1.5 flex items-center gap-2 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text-main)] transition-fast"
      aria-label="Command Palette öffnen (⌘K)"
    >
      <Command className="h-4 w-4" />
      <kbd className="font-mono text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-bg-surface)]/50 border border-[var(--color-wood)]/20">
        ⌘K
      </kbd>
    </button>
  );
}