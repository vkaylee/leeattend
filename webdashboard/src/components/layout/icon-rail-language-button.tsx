/* eslint-disable react/forbid-elements -- internal rail icon buttons with state-driven custom tooltip portals */
import * as Popover from "@radix-ui/react-popover";
import { Check, Globe } from "lucide-react";
import { memo, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import type { LanguageOption, SupportedLanguage } from "@/hooks/use-language";

import { VietnameseFlag, EnglishFlag, ChineseFlag } from "@/components/ui/icons/flags";
import { cn } from "@/lib/utils";

import { TooltipPortal, useTooltipTarget } from "./icon-rail-hooks";

const flagMap: Record<SupportedLanguage, React.ComponentType<{ className?: string }>> = {
  vi: VietnameseFlag,
  en: EnglishFlag,
  zh: ChineseFlag,
};

const SHARED_ICON_CLASSES = "flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative border-0";
const IDLE_CLASSES = "text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-900";

export const IconLanguageButton = memo(function IconLanguageButton({
  currentLanguage,
  changeLanguage,
  languages,
}: {
  currentLanguage: SupportedLanguage;
  changeLanguage: (code: SupportedLanguage) => Promise<void>;
  languages: LanguageOption[];
}) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const { el, hovered, setHovered, refCb } = useTooltipTarget<HTMLButtonElement>();
  const handleBlur = useCallback(() => { setHovered(false); }, [setHovered]);
  const handleFocus = useCallback(() => { setHovered(true); }, [setHovered]);
  const handleMouseEnter = useCallback(() => { setHovered(true); }, [setHovered]);
  const handleMouseLeave = useCallback(() => { setHovered(false); }, [setHovered]);

  const handleSelect = useCallback(async (code: SupportedLanguage) => {
    await changeLanguage(code);
    setOpen(false);
  }, [changeLanguage]);

  const langLabel = currentLanguage.toUpperCase();

  return (
    <Popover.Root onOpenChange={setOpen} open={open}>
      <Popover.Trigger asChild>
        <button
          aria-label={t("appLauncher.language", "Language")}
          className={cn(SHARED_ICON_CLASSES, IDLE_CLASSES)}
          data-testid="nav-rail-lang-toggle"
          onBlur={handleBlur}
          onFocus={handleFocus}
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
          ref={refCb}
          type="button"
        >
          <div className="relative">
            <Globe className="h-4 w-4" />
            <span className="absolute -bottom-1 -right-2 min-w-[15px] h-[13px] px-0.5 rounded-full bg-primary-500 text-white text-[8px] font-black flex items-center justify-center leading-none shadow-sm">
              {langLabel}
            </span>
          </div>
        </button>
      </Popover.Trigger>
      <TooltipPortal el={el} hovered={hovered} label={t("appLauncher.language", "Language")} />

      <Popover.Portal>
        <Popover.Content
          align="start"
          className={cn(
            "z-[10000] min-w-[200px] overflow-hidden rounded-2xl p-1.5",
            "bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xl dark:shadow-black/50",
            "animate-in fade-in-0 zoom-in-95 slide-in-from-left-2 duration-300 ease-out"
          )}
          side="right"
          sideOffset={8}
        >
          <div className="flex flex-col gap-0.5">
            {languages.map((lang) => {
              const LangFlag = flagMap[lang.code];
              const isSelected = currentLanguage === lang.code;
              return (
                <button
                  className={cn(
                    "flex w-full items-center gap-3 rounded-xl px-3.5 py-2.5 text-sm min-h-[44px] cursor-pointer",
                    "transition-all duration-200 active:scale-[0.98]",
                    isSelected
                      ? "bg-primary-50 dark:bg-primary-950/40 text-primary-600 dark:text-primary-400 font-bold border border-primary-100/50 dark:border-primary-900/30"
                      : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800/60 hover:text-slate-900 dark:hover:text-slate-100 border border-transparent"
                  )}
                  data-testid={`rail-lang-option-${lang.code}`}
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
