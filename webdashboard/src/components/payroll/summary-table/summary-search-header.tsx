import { Search } from "lucide-react";
import React from "react";

import { TableDensity, TableDensityToggle } from "@/components/ui/table-density-toggle";

interface SummarySearchHeaderProps {
  density?: TableDensity;
  filteredCount: number;
  onDensityChange?: (density: TableDensity) => void;
  onSearchChange: (value: string) => void;
  searchTerm: string;
  t: (key: string) => string;
}

export function SummarySearchHeader({
  searchTerm,
  onSearchChange,
  filteredCount,
  t,
  density,
  onDensityChange,
}: SummarySearchHeaderProps) {
  return (
    <div className="p-6 border-b border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex flex-col xl:flex-row xl:items-center justify-between gap-6 px-8">
      <div className="flex flex-col md:flex-row md:items-center gap-6 flex-1">
        <div className="relative group flex-1 max-w-lg">
          <Search className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-500 dark:text-slate-400 w-4.5 h-4.5 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
          <input aria-label={t("common.search")} className="w-full pl-14 pr-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[1.5rem] outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm" onChange={(e) => { onSearchChange(e.target.value); }} placeholder={t("payroll.search_placeholder")} type="text" value={searchTerm} />
        </div>
        <div className="flex items-center gap-3">
          <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] bg-white dark:bg-slate-900 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-sm">{filteredCount} {t("common.employees")}</span>
          {density && onDensityChange && (
            <TableDensityToggle density={density} onDensityChange={onDensityChange} />
          )}
        </div>
      </div>
    </div>
  );
}
