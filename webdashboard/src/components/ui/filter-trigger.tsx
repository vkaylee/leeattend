import { ChevronDown } from "lucide-react";
import React, { forwardRef } from "react";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

// ── Types ────────────────────────────────────────────────────────────

export type FilterTriggerVariant = "pill-button" | "icon-button";

export interface FilterTriggerProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "children"> {
  /** Visual variant */
  variant: FilterTriggerVariant;
  /** Accessible label (required for icon-button) */
  label: string;
  /** Custom icon override (default: Filter) */
  icon?: React.ReactNode;
  /** Number shown in the count badge. Omit or pass 0 to hide badge. */
  activeCount?: number;
  /** When true, applies primary/active tint styling */
  hasActive?: boolean;
  /** Show a chevron-down indicator (for dropdown-style triggers) */
  chevron?: boolean;
  /** Custom trigger content (replaces default icon+label layout) */
  children?: React.ReactNode;
}

// ── Internal: Count Badge ────────────────────────────────────────────

function CountBadge({ count, variant }: { count: number; variant: FilterTriggerVariant }) {
  if (count <= 0) return null;

  if (variant === "icon-button") {
    return (
      <span
        key={count}
        className="absolute -top-1 -right-1 min-w-[14px] h-[14px] px-0.5 flex items-center justify-center bg-primary-500 text-white text-[8px] font-bold rounded-full leading-none animate-in zoom-in"
      >
        {count}
      </span>
    );
  }

  return (
    <span
      key={count}
      className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary-600 dark:bg-primary-500 px-1.5 text-[10px] text-white animate-in zoom-in"
    >
      {count}
    </span>
  );
}

// ── Component ────────────────────────────────────────────────────────

/**
 * FilterTrigger — reusable filter toggle button for FilterPanel.
 *
 * Two variants:
 * - `pill-button`: Standalone filter button with icon + optional chevron + inline badge.
 *   Used for toolbar-style filters (DepartmentFilterButton, ShiftTypePicker).
 * - `icon-button`: Compact icon-only button with absolute-positioned badge.
 *   Used for table column header filters (DepartmentFilterHeader, StatusFilterHeader).
 *
 * Spreads remaining HTMLButtonElement props to the underlying Button,
 * so Radix trigger primitives (Popover.Trigger, DropdownMenuTrigger) can
 * attach their event handlers via asChild.
 */
const FilterTrigger = forwardRef<HTMLButtonElement, FilterTriggerProps>(function FilterTrigger(
  {
    variant,
    label,
    icon,
    activeCount = 0,
    hasActive = false,
    chevron = false,
    className,
    children,
    ...restProps
  },
  ref,
) {
  const showBadge = activeCount > 0;

  if (variant === "icon-button") {
    return (
      <Button
        ref={ref}
        aria-label={label}
        className={cn(
          "relative h-5 w-5 rounded transition-colors p-0 border-none shadow-none hover:bg-transparent",
          hasActive
            ? "text-primary-600 dark:text-primary-400"
            : "text-slate-400 dark:text-slate-500 opacity-60 hover:opacity-100 hover:text-primary-600",
          className,
        )}
        title={label}
        type="button"
        variant="ghost"
        {...restProps}
      >
        {icon ?? null}
        <CountBadge count={activeCount} variant="icon-button" />
      </Button>
    );
  }

  // pill-button variant
  return (
    <Button
      ref={ref}
      aria-label={label}
      className={cn(
        "relative flex items-center gap-2 rounded-2xl border px-4 text-sm font-bold uppercase tracking-tight shadow-inner transition-all",
        hasActive || showBadge
          ? "border-primary-500 dark:border-primary-500 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400"
          : "border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:border-primary-200 dark:hover:border-primary-800",
        className,
      )}
      type="button"
      variant="ghost"
      {...restProps}
    >
      {children ?? (
        <>
          {icon ?? null}
          <span className="truncate">{label}</span>
        </>
      )}
      <CountBadge count={activeCount} variant="pill-button" />
      {chevron ? (
        <ChevronDown className="h-3.5 w-3.5 shrink-0 opacity-50" />
      ) : null}
    </Button>
  );
});

FilterTrigger.displayName = "FilterTrigger";

export { FilterTrigger };
export default FilterTrigger;
