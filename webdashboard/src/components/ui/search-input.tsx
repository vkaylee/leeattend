import { Search, X, Loader2 } from 'lucide-react';
import React, { forwardRef, useCallback, useRef } from 'react';

import { cn } from '@/lib/utils';

export interface SearchInputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /** Optional keyboard shortcut label (e.g., "⌘K" or "/") */
  shortcut?: string;
  /** Callback when clear (X) button is clicked */
  onClear?: () => void;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
  /** Extra container styling */
  containerClassName?: string;
  /** Show spinner instead of search icon */
  loading?: boolean;
}

const sizeClasses = {
  sm: 'h-8 px-2.5 text-xs gap-2',
  md: 'h-9 px-3 text-sm gap-2.5',
  lg: 'h-11 px-4 text-base gap-3',
};

const iconSizes = {
  sm: 'h-3.5 w-3.5',
  md: 'h-4 w-4',
  lg: 'h-5 w-5',
};

/**
 * Clean VitePress-inspired SearchInput component
 * Features subtle border, rounded container, clear action, and keyboard badge
 */
export const SearchInput = forwardRef<HTMLInputElement, SearchInputProps>(
  (
    {
      className,
      containerClassName,
      size = 'md',
      shortcut,
      onClear,
      value,
      onChange,
      disabled,
      loading,
      placeholder = 'Search...',
      ...props
    },
    forwardedRef
  ) => {
    const internalRef = useRef<HTMLInputElement>(null);
    const inputRef = (forwardedRef as React.RefObject<HTMLInputElement>) || internalRef;

    const hasValue = value !== undefined ? Boolean(value) : false;

    const handleClear = useCallback(() => {
      if (onClear) {
        onClear();
      } else if (onChange) {
        // Synthesize an empty change event
        const event = {
          target: { value: '' },
          currentTarget: { value: '' },
        } as React.ChangeEvent<HTMLInputElement>;
        onChange(event);
      }
      if (inputRef && 'current' in inputRef && inputRef.current) {
        inputRef.current.focus();
      }
    }, [onClear, onChange, inputRef]);

    return (
      <div
        className={cn(
          'group relative flex w-full items-center rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50/70 dark:bg-slate-900/60 shadow-2xs transition-all duration-200',
          'hover:border-slate-300 dark:hover:border-slate-700 hover:bg-white dark:hover:bg-slate-900',
          'focus-within:border-primary-500 dark:focus-within:border-primary-400 focus-within:ring-2 focus-within:ring-primary-500/20 focus-within:bg-white dark:focus-within:bg-slate-900',
          disabled && 'opacity-60 cursor-not-allowed pointer-events-none',
          sizeClasses[size],
          containerClassName
        )}
      >
        {/* Left search/loading icon */}
        {loading ? (
          <Loader2
            className={cn(
              'shrink-0 animate-spin text-primary-500',
              iconSizes[size]
            )}
          />
        ) : (
          <Search
            className={cn(
              'shrink-0 text-slate-400 dark:text-slate-500 transition-colors group-hover:text-slate-600 dark:group-hover:text-slate-400 group-focus-within:text-primary-600 dark:group-focus-within:text-primary-400',
              iconSizes[size]
            )}
          />
        )}

        {/* Real input element */}
        <input
          className={cn(
            'flex-1 min-w-0 bg-transparent border-0 p-0 font-medium text-slate-800 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-500 outline-none',
            className
          )}
          disabled={disabled}
          onChange={onChange}
          placeholder={placeholder}
          ref={forwardedRef}
          type="search"
          value={value}
          {...props}
        />

        {/* Clear button (when input has value) */}
        {hasValue && (
          <button
            aria-label="Clear search input"
            className="flex h-5 w-5 shrink-0 items-center justify-center rounded-md text-slate-400 hover:text-slate-700 dark:text-slate-500 dark:hover:text-slate-200 hover:bg-slate-200/60 dark:hover:bg-slate-800 transition-colors"
            onClick={handleClear}
            tabIndex={-1}
            type="button"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        )}

        {/* Shortcut badge (e.g. ⌘K or /) */}
        {shortcut && !hasValue && (
          <kbd className="pointer-events-none shrink-0 select-none rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 px-1.5 py-0.5 text-[10px] font-mono font-medium text-slate-400 dark:text-slate-500 shadow-2xs">
            {shortcut}
          </kbd>
        )}
      </div>
    );
  }
);
SearchInput.displayName = 'SearchInput';

export interface SearchTriggerButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Placeholder / label text */
  placeholder?: string;
  /** Shortcut text, defaults to "⌘K" */
  shortcut?: string;
}

/**
 * Clean VitePress-inspired DocSearch trigger button
 * Matches the search button in nginxproxymanager.com header
 */
export const SearchTriggerButton = forwardRef<HTMLButtonElement, SearchTriggerButtonProps>(
  (
    {
      className,
      placeholder = 'Search...',
      shortcut = '⌘K',
      type = 'button',
      ...props
    },
    ref
  ) => (
    <button
      className={cn(
        'group flex h-9 w-full items-center justify-between rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50/80 dark:bg-slate-900/70 px-3 text-xs text-slate-500 dark:text-slate-400 shadow-2xs transition-all duration-200',
        'hover:border-primary-500/60 dark:hover:border-primary-400/60 hover:bg-white dark:hover:bg-slate-900 hover:text-slate-800 dark:hover:text-slate-200',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2',
        className
      )}
      ref={ref}
      type={type}
      {...props}
    >
      <span className="flex items-center gap-2 truncate">
        <Search className="h-3.5 w-3.5 text-slate-400 dark:text-slate-500 transition-colors group-hover:text-primary-500" />
        <span className="truncate font-medium">{placeholder}</span>
      </span>
      {shortcut && (
        <kbd className="shrink-0 rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 px-1.5 py-0.5 text-[10px] font-mono font-semibold text-slate-400 dark:text-slate-500 shadow-2xs">
          {shortcut}
        </kbd>
      )}
    </button>
  )
);
SearchTriggerButton.displayName = 'SearchTriggerButton';
