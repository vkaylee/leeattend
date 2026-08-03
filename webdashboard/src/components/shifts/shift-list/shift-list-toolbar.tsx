import React from "react";
import { useTranslation } from "react-i18next";

import SearchBox from "@/components/common/search-box";
import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { ListFilter } from "./types";

import { FILTER_CHIPS } from "./shift-list-toolbar-constants";

// ── ShiftListToolbar ───────────────────────────────────────────────────

export interface ShiftListToolbarProps {
  activeFilter: ListFilter;
  filteredCount: number;
  onFilterChange: (filter: ListFilter) => void;
  onSearchChange: (query: string) => void;
  searchPlaceholder: string;
  searchQuery: string;
}

/**
 * Toolbar with search input and filter chips for the shift list.
 */
export function ShiftListToolbar({
  searchQuery,
  onSearchChange,
  searchPlaceholder,
  activeFilter,
  onFilterChange,
  filteredCount,
}: ShiftListToolbarProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <span className="font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
          {t("shifts.directory_title")}
        </span>
        <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm">
          {t("common.total").toUpperCase()}: {filteredCount}
        </span>
      </div>

      {/* Search + Filter chips row */}
      <div className="flex flex-col sm:flex-row gap-3 items-start sm:items-center">
        <SearchBox
          className="w-full sm:max-w-xs"
          onChange={onSearchChange}
          placeholder={searchPlaceholder}
          value={searchQuery}
        />
        <div className="flex gap-2 overflow-x-auto scrollbar-hide">
          {FILTER_CHIPS.map((chip) => (
            <Button
              aria-pressed={activeFilter === chip.key}
              className={cn(
                "shrink-0 px-3 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all duration-200 h-auto",
                "border-2 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/50",
                activeFilter === chip.key
                  ? "bg-primary-600 text-white border-primary-600 shadow-md shadow-primary-200 dark:shadow-primary-900/30"
                  : "bg-white dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400",
              )}
              key={chip.key}
              onClick={() => { onFilterChange(chip.key); }}
              type="button"
              variant="ghost"
            >
              {t(chip.labelKey)}
            </Button>
          ))}
        </div>
      </div>
    </div>
  );
}
