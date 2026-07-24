import { Clock, Coffee } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { Popover, PopoverAnchor, PopoverContent } from "@/components/ui/popover";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

/** A single shift interval with optional break */
export interface ShiftIntervalData {
  break_duration_minutes?: number;
  break_end?: string;
  break_start?: string;
  end_time?: string;
  start_time?: string;
}

interface ShiftTimelineScrubberProps {
  className?: string;
  intervals: ShiftIntervalData[];
  isLoading?: boolean;
  shiftName?: string;
  /** compact = small inline (matrix cells), full = standalone with labels */
  variant?: "compact" | "full";
}

/**
 * Parse "HH:mm" → minutes since midnight.
 */
function toMinutes(t: string | undefined): number {
  if (!t) return 0;
  const [h, m] = t.split(":").map(Number);
  return (h || 0) * 60 + (m || 0);
}

/**
 * Determine if a shift is night (crosses midnight or starts late).
 * Returns "night" if start >= 18:00 or end crosses past midnight, else "day".
 */
function classifyShift(
  start: string | undefined,
  end: string | undefined,
): "day" | "night" {
  if (!start || !end) return "day";
  const s = toMinutes(start);
  const e = toMinutes(end);
  // Night shift: starts at 18:00 or later, or is overnight (end < start but actually next day)
  if (s >= 18 * 60 || e < s) return "night";
  return "day";
}

export function ShiftTimelineScrubber({
  intervals,
  shiftName,
  variant = "full",
  isLoading = false,
  className,
}: ShiftTimelineScrubberProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const TOTAL_MINUTES = 24 * 60;

  if (isLoading) {
    return (
      <Skeleton
        className={cn(
          "w-full rounded-xl",
          variant === "compact" ? "h-3" : "h-8",
          className,
        )}
      />
    );
  }

  // Empty state — no intervals
  if (!intervals.length) {
    return (
      <div
        className={cn(
          "w-full rounded-xl bg-slate-100 dark:bg-slate-800/50 border border-dashed border-slate-200 dark:border-slate-700 flex items-center justify-center",
          variant === "compact" ? "h-3" : "h-8",
          className,
        )}
      >
        {variant === "full" && (
          <span className="text-[8px] font-bold uppercase tracking-widest text-slate-300 dark:text-slate-600">
            {t("shifts.monthly.off_day") || "OFF"}
          </span>
        )}
      </div>
    );
  }

  const totalNetMinutes = intervals.reduce((sum, iv) => {
    const s = toMinutes(iv.start_time);
    let e = toMinutes(iv.end_time);
    if (e <= s) e += TOTAL_MINUTES; // overnight shift
    const gross = e - s;
    return sum + Math.max(0, gross - (iv.break_duration_minutes ?? 0));
  }, 0);

  const totalHours = totalNetMinutes / 60;
  const totalLabel =
    totalHours >= 1
      ? `${Math.round(totalHours * 10) / 10}h`
      : `${totalNetMinutes}m`;

  const timelineBar = (
    <div
      aria-label={
        shiftName
          ? `${shiftName}: ${intervals.map((iv) => `${iv.start_time?.substring(0, 5) ?? "--"} - ${iv.end_time?.substring(0, 5) ?? "--"}`).join(", ")}`
          : ""
      }
      className={cn(
        "relative w-full rounded-xl bg-slate-100 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700/60 overflow-hidden",
        variant === "compact" ? "h-3" : "h-8",
        className,
      )}
      role="img"
    >
      {/* Hour markers (full variant only) */}
      {variant === "full" && (
        <div className="absolute inset-0 flex pointer-events-none">
          {Array.from({ length: 25 }).map((_, i) => (
            <div
              className="flex-1 border-l border-slate-200/60 dark:border-slate-700/40"
              key={i}
            >
              {i % 6 === 0 && i > 0 && i < 24 && (
                <span className="absolute -top-4 text-[7px] font-bold text-slate-300 dark:text-slate-600 -translate-x-1/2">
                  {String(i).padStart(2, "0")}
                </span>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Shift blocks */}
      {intervals.map((iv, idx) => {
        const s = toMinutes(iv.start_time);
        let e = toMinutes(iv.end_time);
        if (e <= s) e += TOTAL_MINUTES; // overnight shift spans midnight

        const leftPct = (s / TOTAL_MINUTES) * 100;
        const widthPct = ((e - s) / TOTAL_MINUTES) * 100;
        const shiftClass = classifyShift(iv.start_time, iv.end_time);
        const blockColors =
          shiftClass === "night"
            ? "bg-violet-500/80 dark:bg-violet-500/70 border-violet-400/30 dark:border-violet-400/20"
            : "bg-indigo-500/80 dark:bg-indigo-500/70 border-indigo-400/30 dark:border-indigo-400/20";

        // Break position within this interval
        const bs = iv.break_start ? toMinutes(iv.break_start) : null;
        const be = iv.break_end ? toMinutes(iv.break_end) : null;
        let breakLeftPct = 0;
        let breakWidthPct = 0;
        if (bs !== null && be !== null) {
          let beAdj = be;
          if (beAdj <= bs) beAdj += TOTAL_MINUTES;
          breakLeftPct = ((bs - s) / (e - s)) * 100;
          breakWidthPct = ((beAdj - bs) / (e - s)) * 100;
        }

        return (
          <div
            className={cn(
              "absolute top-0 h-full border-r first:border-l transition-colors",
              blockColors,
            )}
            key={idx}
            style={{
              left: `${leftPct}%`,
              width: `${Math.max(widthPct, 0.5)}%`,
            }}
          >
            {/* Break indicator — diagonal stripe pattern */}
            {bs !== null && be !== null && breakWidthPct > 0 && (
              <div
                className="absolute top-0 h-full"
                style={{
                  left: `${Math.max(breakLeftPct, 0)}%`,
                  width: `${Math.max(breakWidthPct, 1)}%`,
                  backgroundImage:
                    "repeating-linear-gradient(-45deg, transparent, transparent 2px, rgba(255,255,255,0.25) 2px, rgba(255,255,255,0.25) 4px)",
                }}
              />
            )}
          </div>
        );
      })}
    </div>
  );

  // Compact variant — no popover
  if (variant === "compact") {
    return timelineBar;
  }

  // Full variant — wrap in Popover for hover tooltip
  return (
    <Popover modal={false} onOpenChange={setIsOpen} open={isOpen}>
      <PopoverAnchor asChild>
        <button
          className="w-full cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 rounded-xl"
          onBlur={() => {
            setIsOpen(false);
          }}
          onFocus={() => {
            setIsOpen(true);
          }}
          onMouseEnter={() => {
            setIsOpen(true);
          }}
          onMouseLeave={() => {
            setIsOpen(false);
          }}
          type="button"
        >
          {timelineBar}
        </button>
      </PopoverAnchor>
      <PopoverContent
        className="w-72 p-4 rounded-2xl shadow-xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 pointer-events-none animate-in fade-in zoom-in-95 duration-200"
        side="top"
        sideOffset={8}
      >
        <div className="space-y-3">
          {/* Shift name header */}
          {!!shiftName && (
            <div className="border-b border-slate-100 dark:border-slate-800/60 pb-2">
              <span className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight">
                {shiftName}
              </span>
            </div>
          )}

          {/* Interval details */}
          {intervals.map((iv, idx) => {
            const st = iv.start_time?.substring(0, 5) ?? "--";
            const et = iv.end_time?.substring(0, 5) ?? "--";
            const bs = iv.break_start?.substring(0, 5);
            const be = iv.break_end?.substring(0, 5);
            const bd = iv.break_duration_minutes ?? 0;

            return (
              <div className="space-y-1.5" key={idx}>
                <div className="flex items-center gap-2 text-xs text-slate-600 dark:text-slate-350">
                  <Clock
                    className="text-primary-500 dark:text-primary-400 flex-shrink-0"
                    size={13}
                  />
                  <span className="font-semibold">
                    {st} — {et}
                  </span>
                </div>
                {bs && be ? (
                  <div className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 ml-5">
                    <Coffee
                      className="text-amber-500 dark:text-amber-400 flex-shrink-0"
                      size={12}
                    />
                    <span>
                      {t("shifts.table.break_time")}: {bs} — {be} ({bd}
                      m)
                    </span>
                  </div>
                ) : null}
              </div>
            );
          })}

          {/* Total */}
          <div className="border-t border-slate-100 dark:border-slate-800/60 pt-2 flex items-center justify-between">
            <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
              {t("shifts.modal.working_hours") || "Net Working Hours"}
            </span>
            <span className="text-xs font-bold text-primary-600 dark:text-primary-400 tabular-nums">
              {totalLabel}
            </span>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
