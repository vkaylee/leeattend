import { X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";
import { AnimatePresence, motion } from "framer-motion";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

// ── Types ────────────────────────────────────────────────────────────

export interface FilterChipItem {
  /** Unique key for removal */
  key: string;
  /** Display label */
  label: string;
  /** Category name — maps to built-in preset styles */
  category?: string;
  /** Custom icon override */
  icon?: React.ReactNode;
}

export interface ChipStyle {
  icon?: React.ReactNode;
  baseClass: string;
}

export interface FilterChipsProps {
  /** Active filter chips to display */
  chips: FilterChipItem[];
  /** Called when a single chip is removed (X click) */
  onRemove: (key: string) => void;
  /** Called when "Clear all" is clicked */
  onClearAll: () => void;

  /** Per-category style overrides. Falls back to built-in presets, then default. */
  categoryStyles?: Record<string, ChipStyle>;
  /** Show "Clear all" button at end of row */
  showClearAll?: boolean;
  /** Custom "Clear all" label */
  clearAllLabel?: string;
  /** Label shown before chips (e.g. "Đang lọc") */
  label?: string;
  /** Additional CSS */
  className?: string;
  /** Test id prefix */
  "data-testid"?: string;
}

// ── Built-in Category Presets ────────────────────────────────────────

const CATEGORY_PRESETS: Record<string, ChipStyle> = {
  department: {
    baseClass:
      "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800 hover:bg-primary-100 dark:hover:bg-primary-900",
  },
  status: {
    baseClass:
      "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-800 hover:bg-emerald-100 dark:hover:bg-emerald-900",
  },
  custom: {
    baseClass:
      "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-400 border-amber-100 dark:border-amber-800 hover:bg-amber-100 dark:hover:bg-amber-900",
  },
};

const DEFAULT_CHIP_STYLE: ChipStyle = {
  baseClass:
    "bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600",
};

// ── Component ────────────────────────────────────────────────────────

/**
 * FilterChips — row of removable chips showing active filters.
 *
 * Built-in category presets: "department" (primary/blue), "status" (emerald),
 * "custom" (amber). Override via `categoryStyles` prop.
 *
 * Used as the feedback layer in the SelectionFilter system:
 * user selects filters → chips appear → click X to remove or "Clear all" to reset.
 */
export function FilterChips({
  chips,
  onRemove,
  onClearAll,
  categoryStyles,
  showClearAll = true,
  clearAllLabel,
  label,
  className,
  "data-testid": testId,
}: FilterChipsProps) {
  const { t } = useTranslation();

  if (chips.length === 0) return null;

  return (
    <div
      className={cn(
        "flex items-center gap-2 flex-wrap animate-in slide-in-from-top-4 duration-300",
        className,
      )}
      data-testid={testId ?? "filter-chips"}
    >
      {label ? (
        <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mr-1">
          {label}
        </span>
      ) : null}

      <AnimatePresence>
        {chips.map((chip) => {
          const style = (chip.category && categoryStyles?.[chip.category])
            ?? (chip.category ? CATEGORY_PRESETS[chip.category] : undefined)
            ?? DEFAULT_CHIP_STYLE;

          return (
            <motion.div
              key={chip.key}
              initial={{ opacity: 0, y: -8, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -8, scale: 0.95 }}
              transition={{ duration: 0.2 }}
            >
              <Button
                className={cn(
                  "h-auto w-auto px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wide border transition-colors group shadow-none",
                  style.baseClass,
                )}
                data-testid={testId ? `${testId}-chip-${chip.key}` : `filter-chip-${chip.key}`}
                onClick={() => { onRemove(chip.key); }}
                variant="ghost"
              >
                {chip.icon ?? style.icon ? (
                  <span className="opacity-60">{chip.icon ?? style.icon}</span>
                ) : null}
                {chip.label}
                <X className="opacity-40 group-hover:opacity-100 transition-opacity" size={10} />
              </Button>
            </motion.div>
          );
        })}
      </AnimatePresence>

      {showClearAll ? (
        <Button
          className="h-auto w-auto p-0 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-red-500 dark:hover:text-red-400 uppercase tracking-wide transition-colors ml-1 border-none shadow-none hover:bg-transparent"
          data-testid={testId ? `${testId}-clear-all` : "filter-chips-clear-all"}
          onClick={onClearAll}
          variant="ghost"
        >
          {clearAllLabel ?? t("employees.clear_all", "Xóa tất cả")}
        </Button>
      ) : null}
    </div>
  );
}

export default FilterChips;
