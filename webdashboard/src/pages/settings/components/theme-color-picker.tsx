import React from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ThemeColorPickerProps {
  labelClassName?: string;
  onThemeColorChange: (color: string) => void;
  themePrimaryColor: string;
}

export function ThemeColorPicker({
  labelClassName = "text-xs font-semibold text-slate-700 dark:text-slate-300 ml-0.5",
  onThemeColorChange,
  themePrimaryColor,
}: ThemeColorPickerProps) {
  const { t } = useTranslation();

  const presets = [
    { name: t("common.colors.blue"), color: "#2563eb" },
    { name: t("common.colors.primary", { defaultValue: "Chàm" }), color: "#4f46e5" },
    { name: t("common.colors.emerald"), color: "#10b981" },
    { name: t("common.colors.amber"), color: "#d97706" },
    { name: t("common.colors.rose"), color: "#e11d48" },
    { name: t("common.colors.slate"), color: "#475569" },
  ];

  return (
    <div className="space-y-3">
      <label className={labelClassName}>
        {t("common.workspace_settings.primary_color")}
      </label>
      <div className="flex flex-wrap gap-3 items-center">
        {presets.map((preset) => (
          <Button
            className={cn(
              "w-10 h-10 rounded-xl transition-all border-2 flex items-center justify-center shadow-2xs group active:scale-95",
              themePrimaryColor.toLowerCase() === preset.color.toLowerCase()
                ? "border-slate-900 dark:border-white scale-105"
                : "border-transparent hover:scale-105"
            )}
            key={preset.color}
            onClick={() => onThemeColorChange(preset.color)}
            style={{ backgroundColor: preset.color }}
            title={preset.name}
            variant="ghost"
          >
            {themePrimaryColor.toLowerCase() === preset.color.toLowerCase() && (
              <div className="w-2.5 h-2.5 bg-white dark:bg-slate-900 rounded-full shadow-xs" />
            )}
          </Button>
        ))}
        <div className="h-8 w-px bg-slate-200 dark:bg-slate-700 mx-1" />
        <div className="flex items-center gap-2 bg-slate-50 dark:bg-slate-800 px-3 py-1.5 rounded-xl border border-slate-200/80 dark:border-slate-700/80 shadow-2xs">
          <input
            aria-label={t("common.workspace_settings.primary_color")}
            className="w-6 h-6 rounded-md cursor-pointer bg-transparent border-none p-0"
            onChange={(e) => onThemeColorChange(e.target.value)}
            type="color"
            value={themePrimaryColor}
          />
          <input
            aria-label={t("common.workspace_settings.primary_color")}
            className="bg-transparent border-none text-xs font-mono font-semibold text-slate-600 dark:text-slate-300 w-20 outline-none"
            onChange={(e) => onThemeColorChange(e.target.value)}
            type="text"
            value={themePrimaryColor}
          />
        </div>
      </div>
    </div>
  );
}

export default ThemeColorPicker;
