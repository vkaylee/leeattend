import { memo } from "react";
import { Command } from "cmdk";
import { useTranslation } from "react-i18next";
import { CornerDownLeft } from "lucide-react";

import type { FeatureItem } from "@/lib/feature-registry";
import { buildFeatureSearchIndex } from "@/lib/feature-registry";
import { cn } from "@/lib/utils";

interface DsmSearchItemProps {
  item: FeatureItem;
  onSelect: (item: FeatureItem) => void;
}

export const DsmSearchItem = memo(function DsmSearchItem({
  item,
  onSelect,
}: DsmSearchItemProps) {
  const { t } = useTranslation();
  const Icon = item.icon;

  const label = item.titleKey
    ? t(item.titleKey, { defaultValue: item.titleFallback })
    : item.titleFallback;

  const desc = item.descriptionKey
    ? t(item.descriptionKey, { defaultValue: item.descriptionFallback })
    : item.descriptionFallback;

  const searchIndexValue = buildFeatureSearchIndex(item, t);

  const categoryLabels = {
    app: { text: "App", badge: "bg-blue-500/20 text-blue-300 border-blue-500/30" },
    action: { text: "Thao tác", badge: "bg-emerald-500/20 text-emerald-300 border-emerald-500/30" },
    setting: { text: "Cài đặt", badge: "bg-purple-500/20 text-purple-300 border-purple-500/30" },
  };

  const catMeta = categoryLabels[item.category];

  return (
    <Command.Item
      value={searchIndexValue}
      onSelect={() => { onSelect(item); }}
      data-testid={item.testId}
      className={cn(
        "group flex items-center justify-between gap-3 px-3 py-2.5 rounded-xl cursor-pointer select-none",
        "border border-transparent transition-all duration-150",
        "hover:bg-white/10 dark:hover:bg-white/10",
        "data-[selected=true]:bg-white/15 dark:data-[selected=true]:bg-white/20 data-[selected=true]:border-primary-400/40",
        "outline-none",
      )}
    >
      <div className="flex items-center gap-3.5 min-w-0 flex-1">
        {/* Squircle mini */}
        <div
          className={cn(
            "w-9 h-9 rounded-xl flex items-center justify-center shrink-0 bg-gradient-to-br shadow-md border border-white/20",
            item.gradient,
          )}
        >
          <Icon className="w-5 h-5 text-white" />
        </div>

        {/* Text */}
        <div className="flex flex-col min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold text-white truncate group-hover:text-primary-300 transition-colors">
              {label}
            </span>
            <span
              className={cn(
                "px-2 py-0.5 text-[10px] font-medium rounded-full border shrink-0",
                catMeta.badge,
              )}
            >
              {catMeta.text}
            </span>
          </div>
          {desc ? (
            <span className="text-xs text-slate-300 dark:text-slate-400 truncate mt-0.5">
              {desc}
            </span>
          ) : null}
        </div>
      </div>

      {/* Action hint */}
      <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 group-data-[selected=true]:opacity-100 transition-opacity text-slate-300 shrink-0">
        <span className="text-[11px] text-slate-400 hidden sm:inline">Mở</span>
        <CornerDownLeft className="w-3.5 h-3.5 text-primary-400" />
      </div>
    </Command.Item>
  );
});
