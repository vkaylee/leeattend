import * as Popover from '@radix-ui/react-popover';
import { ChevronDown, Check, Globe } from 'lucide-react';
import { useState } from 'react';

import { useLanguage, type SupportedLanguage } from '@/hooks/use-language';
import { cn } from '@/lib/utils';

interface LanguageSwitcherProps {
  className?: string;
  /** Compact mode: show only flag icon */
  compact?: boolean;
}

/**
 * Language switcher dropdown component
 * - Displays current language with flag
 * - Dropdown with all supported languages
 * - Syncs to server on change
 */
export function LanguageSwitcher({ compact = false, className }: LanguageSwitcherProps) {
  const [open, setOpen] = useState(false);
  const { currentOption, languages, changeLanguage } = useLanguage();

  const handleSelect = async (code: SupportedLanguage) => {
    await changeLanguage(code);
    setOpen(false);
  };

  return (
    <Popover.Root onOpenChange={setOpen} open={open}>
      <Popover.Trigger asChild>
        <button
          className={cn(
            'flex items-center gap-2 rounded-xl px-3 py-2.5 text-sm font-medium min-h-[44px]',
            'bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 shadow-sm',
            'text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-white dark:hover:bg-slate-700 hover:border-primary-100 dark:hover:border-primary-800',
            'transition-all duration-200 active:scale-95',
            'focus:outline-none focus:ring-2 focus:ring-primary-500/20',
            className
          )}
          data-testid="language-switcher-trigger"
          type="button"
        >
          {compact ? (
            <Globe size={18} />
          ) : (
            <>
              <span className="text-base">{currentOption.flag}</span>
              <span className="hidden sm:inline font-semibold">{currentOption.nativeName}</span>
            </>
          )}
          <ChevronDown
            className={cn(
              'text-slate-400 dark:text-slate-500 transition-transform duration-200',
              open && 'rotate-180 text-primary-500'
            )}
            size={14}
          />
        </button>
      </Popover.Trigger>

      <Popover.Portal>
        <Popover.Content
          align="end"
          className={cn(
            'z-[10000] min-w-[180px] overflow-hidden rounded-2xl',
            'bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-700 shadow-xl dark:shadow-black/30',
            'animate-in fade-in zoom-in-95 duration-200'
          )}
          sideOffset={8}
        >
          <div className="p-1.5">
            {languages.map((lang) => (
              <button
                className={cn(
                  'flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm min-h-[44px]',
                  'transition-colors duration-150',
                  currentOption.code === lang.code
                    ? 'bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 font-semibold'
                    : 'text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-100'
                )}
                data-testid={`language-option-${lang.code}`}
                key={lang.code}
                onClick={() => handleSelect(lang.code)}
                type="button"
              >
                <span className="text-base">{lang.flag}</span>
                <span className="flex-1 text-left">{lang.nativeName}</span>
                {currentOption.code === lang.code && (
                  <Check className="text-primary-500" size={16} />
                )}
              </button>
            ))}
          </div>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

export default LanguageSwitcher;
