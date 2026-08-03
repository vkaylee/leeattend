import React from "react";

import { cn } from "@/lib/utils";

import type { FilterGroupProps } from "./types";

import { FilterFooter } from "./filter-actions";
import { OptionList } from "./option-list";

// ── Filter Group ─────────────────────────────────────────────────────────

/**
 * FilterGroup — composes the header label, option list (or custom children),
 * and footer actions into a single panel body.
 */
export function FilterGroup({
  className,
  children,
  headerLabel,
  emptyMessage,
  mode,
  onClear,
  onToggle,
  options,
  searchable,
  searchPlaceholder,
  selected,
  showClearButton,
  testId,
  onApply,
  onCancel,
  showApplyButton,
  showSelectionCount,
}: FilterGroupProps) {
  return (
    <div className={cn("flex flex-col", className)}>
      {headerLabel ? (
        <div className="px-3 pt-2 pb-1.5">
          <div className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em]">
            {headerLabel}
          </div>
        </div>
      ) : null}

      {children ?? (
        <OptionList
          emptyMessage={emptyMessage}
          mode={mode}
          onClear={onClear}
          onToggle={onToggle}
          options={options}
          searchable={searchable}
          searchPlaceholder={searchPlaceholder}
          selected={selected}
          showClearButton={showClearButton}
          testId={testId}
        />
      )}

      <FilterFooter
        onApply={onApply}
        onCancel={onCancel}
        onClear={onClear}
        selectedCount={selected.length}
        showApplyButton={showApplyButton}
        showSelectionCount={showSelectionCount}
        totalCount={options.length}
      />
    </div>
  );
}
