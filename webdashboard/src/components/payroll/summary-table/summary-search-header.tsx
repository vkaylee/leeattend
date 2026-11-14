import React from "react";

import { SearchInput } from "@/components/ui/search-input";
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
        <div className="flex-1 max-w-lg">
          <SearchInput
            aria-label={t("common.search")}
            containerClassName="bg-white dark:bg-slate-900 rounded-[1.5rem] h-12 px-4 shadow-sm"
            onChange={(e) => { onSearchChange(e.target.value); }}
            onClear={() => { onSearchChange(""); }}
            placeholder={t("payroll.search_placeholder")}
            size="md"
            value={searchTerm}
          />
        </div>
        <div className="flex items-center gap-3">
          <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] bg-white dark:bg-slate-900 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-sm">{filteredCount} {t("common.employees")}</span>
          {!!density && !!onDensityChange && <TableDensityToggle density={density} onDensityChange={onDensityChange} />}
        </div>
      </div>
    </div>
  );
}
