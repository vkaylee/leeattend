import React from "react";

import { ChevronLeft, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

interface MonthYearPickerProps {
  month: number;
  year: number;
  onChange: (month: number, year: number) => void;
  variant?: "grid" | "list";
  showShortcuts?: boolean;
  className?: string;
}

/**
 * A reusable component for selecting a month and year.
 * Designed with a modern, high-contrast grid for fast interaction.
 */
export function MonthYearPicker({
  month,
  year,
  onChange,
  variant: _variant = "grid",
  showShortcuts = true,
  className,
}: MonthYearPickerProps) {
  const { t } = useTranslation();

  const handleYearChange = (delta: number) => { onChange(month, year + delta); };
  const handleMonthClick = (m: number) => { onChange(m, year); };
  const setLastMonth = () => { const d = new Date(); d.setMonth(d.getMonth() - 1); onChange(d.getMonth() + 1, d.getFullYear()); };
  const setCurrentMonth = () => { const d = new Date(); onChange(d.getMonth() + 1, d.getFullYear()); };

  return (
    <div className={cn("flex flex-col gap-6 p-2 bg-white dark:bg-slate-900 rounded-[2rem]", className)}>
      {/* Year Selector */}
      <div className="flex items-center justify-between bg-slate-100 dark:bg-slate-700/50 p-1 rounded-2xl shadow-inner border border-slate-200 dark:border-slate-700/50">
        <button onClick={(e) => { e.preventDefault(); handleYearChange(-1); }} className="w-10 h-10 flex items-center justify-center hover:bg-white dark:hover:bg-slate-900 rounded-xl text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-all shadow-sm active:scale-90 hover:shadow-md"><ChevronLeft size={20} /></button>
        <span className="text-lg font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase">{year}</span>
        <button onClick={(e) => { e.preventDefault(); handleYearChange(1); }} className="w-10 h-10 flex items-center justify-center hover:bg-white dark:hover:bg-slate-900 rounded-xl text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-all shadow-sm active:scale-90 hover:shadow-md"><ChevronRight size={20} /></button>
      </div>

      {/* Month Grid */}
      <div className="grid grid-cols-3 gap-3">
        {Array.from({ length: 12 }, (_, i) => i + 1).map((m) => (
          <button
            key={m}
            type="button"
            onClick={() => handleMonthClick(m)}
            aria-label={t("common.month_short", { defaultValue: `Tháng ${m}`, month: m })}
            className={cn(
              "py-3.5 rounded-[1.25rem] text-sm font-bold transition-all border",
              month === m
                ? "bg-primary-600 text-white border-primary-600 shadow-lg shadow-primary-100 scale-105 z-10"
                : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:bg-primary-50 dark:bg-primary-950 hover:text-primary-600 dark:text-primary-400 border-slate-100 dark:border-slate-800"
            )}
          >
            {m}
          </button>
        ))}
      </div>

      {/* Shortcuts */}
      {showShortcuts && (
        <div className="flex gap-3 pt-2 border-t border-slate-50 dark:border-slate-800">
          <button type="button" onClick={setLastMonth} className="flex-1 py-3 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-[9px] font-bold uppercase tracking-[0.2em] text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:bg-primary-950 hover:border-primary-100 dark:border-primary-800 transition-all active:scale-95">{t("common.filters.last_month")}</button>
          <button type="button" onClick={setCurrentMonth} className="flex-1 py-3 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-[9px] font-bold uppercase tracking-[0.2em] text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:bg-primary-950 hover:border-primary-100 dark:border-primary-800 transition-all active:scale-95">{t("common.filters.this_month")}</button>
        </div>
      )}
    </div>
  );
}
