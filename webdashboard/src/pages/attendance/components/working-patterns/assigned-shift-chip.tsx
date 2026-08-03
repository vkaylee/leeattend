import { X, Clock } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { getShiftBadge } from "@/components/shifts/shift-badge-style";
import Button from "@/components/ui/button";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Shift } from "@/lib/api";
import {
  getShiftIntervals,
} from "@/lib/shift-card-metrics";
import { cn } from "@/lib/utils";

/** ── Props ──────────────────────────────────────────────────────────── */

interface AssignedShiftChipProps {
  onRemove: () => void;
  shift: Shift;
}

/** ── AssignedShiftChip ──────────────────────────────────────────────── */

/**
 * Chip showing an assigned shift inside the working-pattern grid.
 *
 * Displays the shift name, report symbol, full time range (across all intervals),
 * an accent-coloured left-rail, a compact 24h mini-timeline, and a remove button.
 * Every piece of meaning is duplicated in text — the rail colour is decorative
 * only (WCAG 1.4.1).
 *
 * Removal animates out via a short CSS transition before the item is actually
 * unmounted, which gives smooth exit without needing framer-motion.
 */
export function AssignedShiftChip({ onRemove, shift }: AssignedShiftChipProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();
  const badge = getShiftBadge(shift, t);
  const intervals = getShiftIntervals(shift);
  const leavingTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [isLeaving, setIsLeaving] = useState(false);

  // Clean up the pending timer on unmount so we don't call onRemove twice.
  useEffect(() => {
    return () => {
      if (leavingTimerRef.current) {
        clearTimeout(leavingTimerRef.current);
        leavingTimerRef.current = null;
      }
    };
  }, []);

  const handleRemove = () => {
    if (isLeaving) return;               // debounce double-click
    setIsLeaving(true);
    leavingTimerRef.current = setTimeout(() => {
      leavingTimerRef.current = null;
      onRemove();
    }, 180);
  };

  // ── Build the textual time label (multi-interval aware) ──────────
  const timeLabel = intervals.length === 0
    ? "—"
    : intervals
        .map((iv) => `${iv.start_time}–${formatTime(iv.end_time)}`)
        .join(" · ");

  const showSplitMarker = intervals.length > 1;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 pl-0 pr-1.5 py-1 rounded-xl border shadow-sm",
        "transition-all duration-180 ease-out",
        badge.bg,
        badge.text,
        badge.accent.includes("sky") ? "border-sky-200 dark:border-sky-800/60" :
        badge.accent.includes("amber") ? "border-amber-200 dark:border-amber-800/60" :
        badge.accent.includes("indigo") ? "border-indigo-200 dark:border-indigo-800/60" :
        "border-teal-200 dark:border-teal-800/60",
        isLeaving
          ? "animate-out fade-out zoom-out-95 duration-180 opacity-0 scale-95"
          : "animate-in fade-in zoom-in-95 slide-in-from-bottom-1 duration-350",
      )}
    >
      {/* Accent left-rail — decorative, meaning is duplicated in the text. */}
      <span
        aria-hidden="true"
        className={cn("self-stretch w-1 rounded-l-lg -mr-0.5", badge.accent)}
      />

      {/* Shift name + symbol */}
      <span className="flex items-center gap-1">
        <Clock aria-hidden="true" className="opacity-60 flex-shrink-0" size={10} />
        <span className="text-[10px] font-extrabold uppercase tracking-tight truncate max-w-[10ch]">
          {shift.name}
        </span>
      </span>

      {!!shift.report_symbol && (
        <span className="rounded-sm bg-white/60 dark:bg-black/30 px-1 py-px font-mono text-[8px] font-bold opacity-80 flex-shrink-0">
          {shift.report_symbol}
        </span>
      )}

      {/* Time range (compact, split-aware) */}
      <span className="text-[9px] font-bold tabular-nums opacity-70 whitespace-nowrap">
        {timeLabel}
      </span>

      {!!showSplitMarker && <span className="text-[8px] font-extrabold opacity-50">&nbsp;{t("shifts.picker.card.blocks", { count: intervals.length })}</span>}

      {/* Remove button */}
      <Button
        aria-label={[t("common.remove"), shift.name].filter(Boolean).join(": ")}
        className={cn(
          "!h-auto !min-w-0 !p-0.5 !rounded-md flex-shrink-0 !gap-0",
          "opacity-50 hover:opacity-100 hover:bg-rose-100 dark:hover:bg-rose-950/50 hover:text-rose-600 dark:hover:text-rose-400",
          "focus:outline-none focus-visible:ring-1 focus-visible:ring-rose-400",
        )}
        onClick={handleRemove}
        type="button"
        variant="ghost"
      >
        <X size={11} />
      </Button>
    </span>
  );
}
