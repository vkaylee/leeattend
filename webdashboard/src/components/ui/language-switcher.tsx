import * as Popover from '@radix-ui/react-popover';
import { ChevronDown, Check, Globe } from 'lucide-react';
import { memo, useCallback, useState } from 'react';

import { useLanguage, type SupportedLanguage } from '@/hooks/use-language';
import { cn } from '@/lib/utils';

import { VietnameseFlag, EnglishFlag, ChineseFlag } from './icons/flags';

interface LanguageSwitcherProps {
  className?: string;
  /** Compact mode: show only flag icon */
  compact?: boolean;
}

const flagMap: Record<SupportedLanguage, React.ComponentType<{ className?: string }>> = {
  vi: VietnameseFlag,
  en: EnglishFlag,
  zh: ChineseFlag,
};

/**
 * Language switcher dropdown component
 * - Displays current language with flag
 * - Dropdown with all supported languages
 * - Syncs to server on change
 */
export const LanguageSwitcher = memo(function LanguageSwitcher({ compact = false, className }: LanguageSwitcherProps) {
  const [open, setOpen] = useState(false);
  const { currentOption, languages, changeLanguage } = useLanguage();

  const handleSelect = useCallback(async (code: SupportedLanguage) => {
    await changeLanguage(code);
    setOpen(false);
  }, [changeLanguage]);

  const CurrentFlag = flagMap[currentOption.code];

  return (
    <Popover.Root onOpenChange={setOpen} open={open}>
      <Popover.Trigger asChild>
        <button
          className={cn(
            'flex items-center gap-2.5 rounded-xl px-4 py-2 text-sm font-semibold min-h-[44px] cursor-pointer',
            'bg-slate-50/50 dark:bg-slate-800/50 border border-slate-200/50 dark:border-slate-700/50 shadow-sm backdrop-blur-md',
            'text-slate-700 dark:text-slate-200 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-white dark:hover:bg-slate-800 hover:border-primary-200 dark:hover:border-primary-800/80',
            'transition-all duration-300 active:scale-95',
            'focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-400 dark:focus:border-primary-600',
            className
          )}
          data-testid="language-switcher-trigger"
          type="button"
        >
          {compact ? (
            <Globe size={18} />
          ) : (
            <>
              <CurrentFlag className="w-5 h-5" />
              <span className="hidden sm:inline font-semibold">{currentOption.nativeName}</span>
            </>
          )}
          <ChevronDown
            className={cn(
              'text-slate-400 dark:text-slate-500 transition-transform duration-300',
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
            'z-[10000] min-w-[200px] overflow-hidden rounded-2xl p-1.5',
            'bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xl dark:shadow-black/50',
            'animate-in fade-in-0 zoom-in-95 slide-in-from-top-2 duration-300 ease-out'
          )}
          sideOffset={8}
        >
          <div className="flex flex-col gap-0.5">
            {languages.map((lang) => {
              const LangFlag = flagMap[lang.code];
              const isSelected = currentOption.code === lang.code;
              return (
                <button
                  className={cn(
                    'flex w-full items-center gap-3 rounded-xl px-3.5 py-2.5 text-sm min-h-[44px] cursor-pointer',
                    'transition-all duration-200 active:scale-[0.98]',
                    isSelected
                      ? 'bg-primary-50 dark:bg-primary-950/40 text-primary-600 dark:text-primary-400 font-bold border border-primary-100/50 dark:border-primary-900/30'
                      : 'text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800/60 hover:text-slate-900 dark:hover:text-slate-100 border border-transparent'
                  )}
                  data-testid={`language-option-${lang.code}`}
                  key={lang.code}
                  onClick={() => handleSelect(lang.code)}
                  type="button"
                >
                  <LangFlag className="w-5 h-5 shadow-sm" />
                  <div className="flex-1 text-left flex flex-col">
                    <span className="leading-tight">{lang.nativeName}</span>
                    <span className="text-[10px] text-slate-400 dark:text-slate-500 font-medium leading-none mt-0.5">
                      {lang.name}
                    </span>
                  </div>
                  {isSelected ? <Check className="text-primary-500 dark:text-primary-400 flex-shrink-0" size={16} strokeWidth={2.5} /> : null}
                </button>
              );
            })}
          </div>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
});

LanguageSwitcher.displayName = "LanguageSwitcher";

export default LanguageSwitcher;
