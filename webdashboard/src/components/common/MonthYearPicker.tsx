import React from "react";
import { useTranslation } from "react-i18next";
import { ChevronLeft, ChevronRight, ArrowLeft, ArrowRight } from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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

  const handleYearChange = (delta: number) => {
    onChange(month, year + delta);
  };

  const handleMonthClick = (m: number) => {
    onChange(m, year);
  };

  const setLastMonth = () => {
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    onChange(d.getMonth() + 1, d.getFullYear());
  };

  const setCurrentMonth = () => {
    const d = new Date();
    onChange(d.getMonth() + 1, d.getFullYear());
  };

  return (
    <div className={cn("flex flex-col gap-4", className)}>
      {/* Year Selector */}
      <div className="flex items-center justify-between bg-white p-2 rounded-xl shadow-sm border border-slate-100">
        <button
          onClick={(e) => {
            e.preventDefault();
            handleYearChange(-1);
          }}
          className="p-2 hover:bg-slate-50 rounded-lg text-slate-400 hover:text-slate-900 transition-colors"
        >
          <ChevronLeft className="w-5 h-5" />
        </button>
        <span className="text-lg font-black text-slate-900">{year}</span>
        <button
          onClick={(e) => {
            e.preventDefault();
            handleYearChange(1);
          }}
          className="p-2 hover:bg-slate-50 rounded-lg text-slate-400 hover:text-slate-900 transition-colors"
        >
          <ChevronRight className="w-5 h-5" />
        </button>
      </div>

      {/* Month Grid */}
      <div className="grid grid-cols-3 sm:grid-cols-4 gap-2">
        {Array.from({ length: 12 }, (_, i) => i + 1).map((m) => (
          <button
            key={m}
            type="button"
            onClick={() => handleMonthClick(m)}
            className={cn(
              "py-2.5 rounded-xl text-sm font-bold transition-all border",
              month === m
                ? "bg-slate-900 text-white border-slate-900 shadow-md shadow-slate-200 scale-105 z-10"
                : "bg-white text-slate-500 hover:bg-slate-50 hover:text-slate-900 border-slate-100"
            )}
          >
            {t("common.month_short", { defaultValue: `Tháng ${m}`, month: m })}
          </button>
        ))}
      </div>

      {/* Shortcuts */}
      {showShortcuts && (
        <div className="flex gap-2 pt-2 border-t border-slate-100">
          <button
            type="button"
            onClick={setLastMonth}
            className="flex-1 py-2 rounded-xl bg-white border border-slate-200 text-xs font-bold text-slate-500 hover:text-slate-900 hover:border-slate-300 transition-all"
          >
            {t("common.filters.last_month", { defaultValue: "Tháng trước" })}
          </button>
          <button
            type="button"
            onClick={setCurrentMonth}
            className="flex-1 py-2 rounded-xl bg-white border border-slate-200 text-xs font-bold text-slate-500 hover:text-slate-900 hover:border-slate-300 transition-all"
          >
            {t("common.filters.this_month", { defaultValue: "Tháng này" })}
          </button>
        </div>
      )}
    </div>
  );
}
