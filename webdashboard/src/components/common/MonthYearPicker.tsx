import React from "react";
import { ChevronLeft, ChevronRight, Calendar } from "lucide-react";
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
  variant = "grid",
  showShortcuts = true,
  className,
}: MonthYearPickerProps) {
  const { t } = useTranslation();

  const handleYearChange = (delta: number) => { onChange(month, year + delta); };
  const handleMonthClick = (m: number) => { onChange(m, year); };
  const setLastMonth = () => { const d = new Date(); d.setMonth(d.getMonth() - 1); onChange(d.getMonth() + 1, d.getFullYear()); };
  const setCurrentMonth = () => { const d = new Date(); onChange(d.getMonth() + 1, d.getFullYear()); };

  return (
    <div className={cn("flex flex-col gap-6 p-2 bg-white rounded-[2rem]", className)}>
      {/* Year Selector */}
      <div className="flex items-center justify-between bg-slate-100/50 p-1 rounded-2xl shadow-inner border border-slate-200/50">
        <button onClick={(e) => { e.preventDefault(); handleYearChange(-1); }} className="w-10 h-10 flex items-center justify-center hover:bg-white rounded-xl text-slate-400 hover:text-primary-600 transition-all shadow-sm active:scale-90 hover:shadow-md"><ChevronLeft size={20} /></button>
        <span className="text-lg font-black text-slate-900 tracking-tighter uppercase">{year}</span>
        <button onClick={(e) => { e.preventDefault(); handleYearChange(1); }} className="w-10 h-10 flex items-center justify-center hover:bg-white rounded-xl text-slate-400 hover:text-primary-600 transition-all shadow-sm active:scale-90 hover:shadow-md"><ChevronRight size={20} /></button>
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
              "py-3.5 rounded-[1.25rem] text-sm font-black transition-all border",
              month === m
                ? "bg-primary-600 text-white border-primary-600 shadow-lg shadow-primary-100 scale-105 z-10"
                : "bg-white text-slate-500 hover:bg-primary-50 hover:text-primary-600 border-slate-100"
            )}
          >
            {m}
          </button>
        ))}
      </div>

      {/* Shortcuts */}
      {showShortcuts && (
        <div className="flex gap-3 pt-2 border-t border-slate-50">
          <button type="button" onClick={setLastMonth} className="flex-1 py-3 rounded-xl bg-slate-50 border border-slate-100 text-[9px] font-black uppercase tracking-[0.2em] text-slate-400 hover:text-primary-600 hover:bg-primary-50 hover:border-primary-100 transition-all active:scale-95">{t("common.filters.last_month")}</button>
          <button type="button" onClick={setCurrentMonth} className="flex-1 py-3 rounded-xl bg-slate-50 border border-slate-100 text-[9px] font-black uppercase tracking-[0.2em] text-slate-400 hover:text-primary-600 hover:bg-primary-50 hover:border-primary-100 transition-all active:scale-95">{t("common.filters.this_month")}</button>
        </div>
      )}
    </div>
  );
}
