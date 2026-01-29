import { useState } from 'react';
import { ChevronDown, Check, Globe } from 'lucide-react';
import * as Popover from '@radix-ui/react-popover';
import { cn } from '@/lib/utils';
import { useLanguage, type SupportedLanguage } from '@/hooks/use-language';

interface LanguageSwitcherProps {
  /** Compact mode: show only flag icon */
  compact?: boolean;
  className?: string;
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
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger asChild>
        <button
          type="button"
          data-testid="language-switcher-trigger"
          className={cn(
            'flex items-center gap-2 rounded-xl px-3 py-2.5 text-sm font-medium',
            'bg-slate-50 border border-slate-100 shadow-sm',
            'text-slate-600 hover:text-primary-600 hover:bg-white hover:border-primary-100',
            'transition-all duration-200 active:scale-95',
            'focus:outline-none focus:ring-2 focus:ring-primary-500/20',
            className
          )}
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
            size={14}
            className={cn(
              'text-slate-400 transition-transform duration-200',
              open && 'rotate-180 text-primary-500'
            )}
          />
        </button>
      </Popover.Trigger>

      <Popover.Portal>
        <Popover.Content
          className={cn(
            'z-[10000] min-w-[180px] overflow-hidden rounded-2xl',
            'bg-white border border-slate-100 shadow-xl',
            'animate-in fade-in zoom-in-95 duration-200'
          )}
          sideOffset={8}
          align="end"
        >
          <div className="p-1.5">
            {languages.map((lang) => (
              <button
                key={lang.code}
                type="button"
                data-testid={`language-option-${lang.code}`}
                onClick={() => handleSelect(lang.code)}
                className={cn(
                  'flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm',
                  'transition-colors duration-150',
                  currentOption.code === lang.code
                    ? 'bg-primary-50 text-primary-600 font-semibold'
                    : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900'
                )}
              >
                <span className="text-base">{lang.flag}</span>
                <span className="flex-1 text-left">{lang.nativeName}</span>
                {currentOption.code === lang.code && (
                  <Check size={16} className="text-primary-500" />
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
