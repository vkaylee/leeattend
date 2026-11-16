import Button from "@/components/ui/button";
import {
  Globe,
  Check,
} from "lucide-react";
import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

import { VietnameseFlag, EnglishFlag, ChineseFlag } from "../ui/icons/flags";

type AuthLang = "vi" | "en" | "zh";

const AUTH_LANGUAGES: { code: AuthLang; nativeName: string; Flag: React.ComponentType<{ className?: string }> }[] = [
  { code: "vi", nativeName: "Tiếng Việt", Flag: VietnameseFlag },
  { code: "en", nativeName: "English",    Flag: EnglishFlag },
  { code: "zh", nativeName: "中文",        Flag: ChineseFlag },
];

function normalizeAuthLang(lang: string): AuthLang {
  const supported: AuthLang[] = ["vi", "en", "zh"];
  const base = lang.split("-")[0] as AuthLang;
  return supported.includes(base) ? base : "vi";
}

export function AuthLanguageSwitcher() {
  const { i18n } = useTranslation();
  const [open, setOpen] = useState(false);
  const current = normalizeAuthLang(i18n.language);
  const currentDef = AUTH_LANGUAGES.find((l) => l.code === current) ?? AUTH_LANGUAGES[0];

  const handleSelect = useCallback(
    async (code: AuthLang) => {
      await i18n.changeLanguage(code);
      localStorage.setItem("i18nextLng", code);
      setOpen(false);
    },
    [i18n]
  );

  return (
    <div className="relative">
      <Button
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label="Switch language"
        className={cn(
          "flex items-center gap-2 rounded-xl !px-3 py-2 text-sm font-semibold !min-h-[40px] !h-auto cursor-pointer",
          "bg-slate-100/80 dark:bg-slate-800/60 border border-slate-200/60 dark:border-slate-700/50",
          "text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400",
          "hover:bg-white dark:hover:bg-slate-800 hover:border-primary-200 dark:hover:border-primary-700",
          "transition-all duration-200 active:scale-95",
          "focus:outline-none focus:ring-2 focus:ring-primary-500/20"
        )}
        data-testid="auth-language-switcher"
        onClick={() => { setOpen((v) => !v); }}
        type="button"
        size="sm"
        variant="ghost"
      >
        <currentDef.Flag className="w-4 h-4" />
        <span className="hidden sm:inline">{currentDef.nativeName}</span>
        <Globe className="sm:hidden w-4 h-4 text-slate-400" />
      </Button>

      {open ? (
        <>
          <div
            aria-hidden="true"
            className="fixed inset-0 z-[9998]"
            onClick={() => { setOpen(false); }}
          />
          <div
            className={cn(
              "absolute right-0 top-full mt-2 z-[9999] min-w-[180px] rounded-2xl p-1.5",
              "bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl",
              "border border-slate-200/80 dark:border-slate-800/80 shadow-2xl",
              "animate-in fade-in-0 zoom-in-95 slide-in-from-top-2 duration-200"
            )}
            role="listbox"
          >
            {AUTH_LANGUAGES.map(({ code, nativeName, Flag }) => {
              const isSelected = current === code;
              return (
                <Button
                  aria-selected={isSelected}
                  className={cn(
                    "flex w-full items-center gap-3 rounded-xl !px-3 py-2.5 text-sm !min-h-[44px] !h-auto cursor-pointer transition-all duration-150",
                    isSelected
                      ? "bg-primary-50 dark:bg-primary-950/40 text-primary-600 dark:text-primary-400 font-bold"
                      : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800/60"
                  )}
                  data-testid={`auth-lang-${code}`}
                  key={code}
                  onClick={() => { void handleSelect(code); }}
                  role="option"
                  type="button"
                  size="sm"
                  variant="ghost"
                >
                  <Flag className="w-4 h-4" />
                  <span className="flex-1 text-left">{nativeName}</span>
                  {isSelected ? <Check className="text-primary-500" size={14} strokeWidth={2.5} /> : null}
                </Button>
              );
            })}
          </div>
        </>
      ) : null}
    </div>
  );
}
