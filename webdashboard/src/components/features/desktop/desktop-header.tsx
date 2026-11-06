import { Search, SlidersHorizontal, Sparkles, Sun, Moon, Sunset } from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { openDsmLauncher } from "@/components/features/dsm-launcher";
import Button from "@/components/ui/button";
import { useProfile } from "@/hooks/use-profile";

interface DesktopHeaderProps {
  onOpenCustomizer: () => void;
}

export function DesktopHeader({ onOpenCustomizer }: DesktopHeaderProps) {
  const { t } = useTranslation();
  const { user } = useProfile();

  const greeting = useMemo(() => {
    const hour = new Date().getHours();
    if (hour < 12) {
      return {
        text: t("desktop.goodMorning", "Chào buổi sáng"),
        icon: Sun,
        color: "text-amber-500",
      };
    }
    if (hour < 18) {
      return {
        text: t("desktop.goodAfternoon", "Chào buổi chiều"),
        icon: Sunset,
        color: "text-orange-500",
      };
    }
    return {
      text: t("desktop.goodEvening", "Chào buổi tối"),
      icon: Moon,
      color: "text-indigo-400",
    };
  }, [t]);

  const GreetingIcon = greeting.icon;
  const displayName = user?.full_name || user?.email?.split("@")[0] || "đồng nghiệp";

  return (
    <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4 pb-2">
      {/* Left: User Welcome */}
      <div className="space-y-1">
        <div className="flex items-center gap-2">
          <GreetingIcon className={`w-5 h-5 ${greeting.color}`} />
          <h1 className="text-xl sm:text-2xl font-black tracking-tight text-slate-800 dark:text-slate-100">
            {greeting.text},{" "}
            <span className="text-primary-600 dark:text-primary-400">{displayName}</span>
          </h1>
        </div>
        <p className="text-xs sm:text-sm text-slate-500 dark:text-slate-400">
          {t("desktop.subtitle", "Không gian làm việc & Bảng điều khiển Synology DSM 7")}
        </p>
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-3">
        {/* Spotlight Trigger */}
        <Button
          type="button"
          variant="outline"
          onClick={openDsmLauncher}
          data-testid="desktop-search-trigger"
          className="flex items-center gap-2.5 px-4 h-10 rounded-2xl bg-slate-100/80 dark:bg-slate-800/80 hover:bg-slate-200/80 dark:hover:bg-slate-700/80 border border-slate-200/60 dark:border-slate-700/60 text-xs text-slate-600 dark:text-slate-300 transition-all shadow-2xs group"
        >
          <Search className="w-4 h-4 text-primary-500 group-hover:scale-110 transition-transform" />
          <span className="font-medium hidden sm:inline">
            {t("desktop.searchApps", "Tìm ứng dụng & tính năng...")}
          </span>
          <kbd className="px-2 py-0.5 text-[10px] font-semibold text-slate-400 bg-white/60 dark:bg-black/40 rounded-lg border border-slate-200 dark:border-slate-700">
            ⌘K
          </kbd>
        </Button>

        {/* Customize Desktop Button */}
        <Button
          type="button"
          onClick={onOpenCustomizer}
          variant="secondary"
          className="h-10 px-4 rounded-2xl text-xs font-bold flex items-center gap-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-2xs hover:shadow-xs"
          data-testid="desktop-customize-btn"
        >
          <SlidersHorizontal className="w-4 h-4 text-primary-500" />
          <span>{t("desktop.customizeWidgets", "Gắn Widget")}</span>
        </Button>
      </div>
    </div>
  );
}
