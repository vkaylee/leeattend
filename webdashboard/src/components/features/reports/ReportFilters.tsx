import React from "react";
import { Calendar, CalendarRange, Search, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface ReportFiltersProps {
  filterMode: "this_month" | "last_month" | "custom" | "specific_month";
  dateRange: { start: string; end: string };
  specificMonth: { month: number; year: number };
  searchQuery: string;
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  t: (key: string) => string;
  onFilterModeChange: (mode: "this_month" | "last_month" | "custom" | "specific_month") => void;
  onDateRangeChange: (range: { start: string; end: string }) => void;
  onSpecificMonthChange: (month: number, year: number) => void;
  onSearchQueryChange: (query: string) => void;
}

export default function ReportFilters({
  filterMode,
  dateRange,
  specificMonth,
  searchQuery,
  searchInputRef,
  t,
  onFilterModeChange,
  onDateRangeChange,
  onSpecificMonthChange,
  onSearchQueryChange,
}: ReportFiltersProps) {
  return (
    <>
      <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-8">

        <div className="flex flex-col md:flex-row md:items-center gap-6 bg-white dark:bg-slate-900 p-3 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
          <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
            {["this_month", "last_month"].map(mode => (
              <button
                key={mode}
                onClick={() => onFilterModeChange(mode as any)}
                className={cn(
                  "px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
                  filterMode === mode
                    ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                    : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
                )}
              >
                {t(`common.filters.${mode}`).toUpperCase()}
              </button>
            ))}

            <div className="h-6 w-px bg-slate-200 dark:bg-slate-600 mx-2 opacity-50" />

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  className={cn(
                    "flex items-center gap-2 px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
                    filterMode === "specific_month"
                      ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                      : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
                  )}
                >
                  <CalendarRange size={14} />
                  {filterMode === "specific_month"
                    ? `${specificMonth.month}/${specificMonth.year}`
                    : t("common.filters.specific_month").toUpperCase()}
                </button>
              </DropdownMenuTrigger>

              <DropdownMenuContent
                className="w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/95 backdrop-blur-xl"
                align="end"
                sideOffset={12}
              >
                <MonthYearPicker
                  month={specificMonth.month}
                  year={specificMonth.year}
                  onChange={onSpecificMonthChange}
                />
              </DropdownMenuContent>
            </DropdownMenu>

            <button
              onClick={() => onFilterModeChange("custom")}
              className={cn(
                "px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
                filterMode === "custom"
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
              )}
            >
              {t("common.filters.custom").toUpperCase()}
            </button>
          </div>
        </div>
      </div>

      {/* Search input — standalone row for prominence */}
      <div className="relative">
        <Search size={18} className="absolute left-6 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500 pointer-events-none" />
        <input
          ref={searchInputRef}
          autoFocus
          type="text"
          value={searchQuery}
          onChange={(e) => onSearchQueryChange(e.target.value)}
          placeholder={t("reports.search_placeholder")}
          className="w-full pl-14 pr-12 py-4 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl text-sm font-bold text-slate-700 dark:text-slate-200 placeholder:text-slate-400 dark:placeholder:text-slate-500 placeholder:font-medium focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 dark:focus:border-primary-700 transition-all shadow-sm uppercase tracking-tight"
        />
        {searchQuery && (
          <button
            onClick={() => onSearchQueryChange("")}
            className="absolute right-4 top-1/2 -translate-y-1/2 p-1.5 rounded-lg text-slate-400 hover:text-rose-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-all active:scale-90"
          >
            <X size={16} />
          </button>
        )}
      </div>

      {filterMode === "custom" && (
        <div className="bg-white dark:bg-slate-900 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-primary-900/5 grid grid-cols-1 md:grid-cols-2 gap-10 animate-in slide-in-from-top-4 duration-500 max-w-2xl">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
              <Calendar size={12} /> {t("common.from_date")}
            </label>
            <input
              type="date"
              value={dateRange.start}
              onChange={(e) => onDateRangeChange({ ...dateRange, start: e.target.value })}
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-sm shadow-inner"
            />
          </div>
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
              <Calendar size={12} /> {t("common.to_date")}
            </label>
            <input
              type="date"
              value={dateRange.end}
              onChange={(e) => onDateRangeChange({ ...dateRange, end: e.target.value })}
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-sm shadow-inner"
            />
          </div>
        </div>
      )
      }
    </>
  );
}
