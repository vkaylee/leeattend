import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import {
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

import type { FilterMode, FilterOption } from "./types";

import { DropdownActions } from "./filter-actions";
import { MultiOptionRow, SingleOptionRow } from "./filter-field-renderer";

// ── Dropdown Content ─────────────────────────────────────────────────────

interface FilterDropdownContentProps {
  align: "start" | "center" | "end";
  effectiveSelected: string[];
  mode: FilterMode;
  onApply: () => void;
  onCancel: () => void;
  onClear: () => void;
  onDropdownToggle: (value: string) => void;
  options: FilterOption[];
  showApplyButton: boolean;
  testId?: string;
}

/**
 * FilterDropdownContent — renders the option list and actions inside a
 * DropdownMenuContent. Used by FilterPanel for the "dropdown" container.
 */
export function FilterDropdownContent({
  effectiveSelected,
  mode,
  onClear,
  onDropdownToggle,
  onApply,
  onCancel,
  options,
  showApplyButton,
  testId,
}: FilterDropdownContentProps) {
  const { t } = useTranslation();

  return (
    <>
      {/* All / reset item (multi mode, live only) */}
      {mode === "multi" && !showApplyButton ? (
        <>
          <Button
            className={cn(
              "h-auto w-full px-3 py-2.5 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide transition-colors border-none shadow-none justify-start",
              effectiveSelected.length === 0
                ? "text-primary-600 dark:text-primary-400"
                : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800",
            )}
            data-testid={testId ? `${testId}-option-all` : undefined}
            onClick={(e) => { e.preventDefault(); e.stopPropagation(); onClear(); }}
            type="button"
            variant="ghost"
          >
            {t("common.all_departments", "All")}
          </Button>
          <DropdownMenuSeparator />
        </>
      ) : null}

      {options.map((opt) => {
        const checked = effectiveSelected.includes(opt.value);
        const isMultiLive = mode === "multi" && !showApplyButton;
        if (mode === "multi") {
          return (
            <MultiOptionRow
              checked={checked}
              key={opt.value}
              onToggle={() => { onDropdownToggle(opt.value); }}
              option={opt}
              stopPropagation={isMultiLive}
              testId={testId}
            />
          );
        }
        return (
          <SingleOptionRow
            checked={checked}
            key={opt.value}
            onSelect={() => { onDropdownToggle(opt.value); }}
            option={opt}
            testId={testId}
          />
        );
      })}

      <DropdownActions
        effectiveSelected={effectiveSelected}
        onApply={onApply}
        onCancel={onCancel}
        onClear={onClear}
        showApplyButton={showApplyButton}
        testId={testId}
      />
    </>
  );
}
