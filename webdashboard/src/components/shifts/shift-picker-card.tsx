import Button from "@/components/ui/button";
import type { TFunction } from "i18next";

import { Coffee, Clock, Moon, Star, SplitSquareHorizontal } from "lucide-react";
import { forwardRef } from "react";

import type { ShiftCardMetrics } from "@/lib/shift-card-metrics";

import { Shift } from "@/lib/api";
import { formatDurationShort } from "@/lib/shift-card-metrics";
import { cn } from "@/lib/utils";

import type { ShiftBadgeStyle } from "./shift-badge-style";

import { ShiftCardTimeline } from "./shift-card-timeline";

// ── Time range row ───────────────────────────────────────────────────

interface TimeRowProps {
  formatTime: (v: string) => string;
  metrics: ShiftCardMetrics;
  t: TFunction;
}

/**
 * Every interval of the shift, comma-separated. A split shift shows
 * "08:00–12:00 · 14:00–18:00" instead of silently hiding its second block.
 */
function TimeRow({ formatTime, metrics, t }: TimeRowProps) {
  return (
    <div className="flex items-center gap-1.5 text-xs font-bold text-slate-600 dark:text-slate-300">
      <Clock aria-hidden="true" className="shrink-0 text-slate-400 dark:text-slate-500" size={12} />
      <span className="flex flex-wrap items-center gap-x-1.5 gap-y-0.5 tabular-nums">
        {metrics.intervals.map((iv, index) => (
          <span className="whitespace-nowrap" key={`${iv.startTime}-${iv.endTime}-${String(index)}`}>
            {index > 0 ? (
              <span aria-hidden="true" className="mr-1.5 text-slate-300 dark:text-slate-600">
                ·
              </span>
            ) : null}
            {iv.startTime ? formatTime(iv.startTime) : "—"}
            <span aria-hidden="true" className="mx-0.5 text-slate-400 dark:text-slate-500">
              –
            </span>
            {iv.endTime ? formatTime(iv.endTime) : "—"}
            {iv.crossesMidnight ? (
              <span
                className="ml-1 align-super text-[9px] font-bold text-indigo-500 dark:text-indigo-400"
                title={t("shifts.picker.card.next_day")}
              >
                +1
              </span>
            ) : null}
          </span>
        ))}
      </span>
    </div>
  );
}

// ── Break row ────────────────────────────────────────────────────────

function BreakRow({
  formatTime,
  metrics,
  t,
}: {
  formatTime: (v: string) => string;
  metrics: ShiftCardMetrics;
  t: TFunction;
}) {
  if (metrics.totalBreakMinutes <= 0) return null;

  const fixed = metrics.intervals.find((iv) => iv.isFixedBreak);
  const label = fixed
    ? `${formatTime(fixed.breakStart)}–${formatTime(fixed.breakEnd)}`
    : formatDurationShort(metrics.totalBreakMinutes);

  return (
    <span className="inline-flex items-center gap-1 text-[11px] font-bold text-amber-700 dark:text-amber-400">
      <Coffee aria-hidden="true" className="shrink-0" size={11} />
      <span className="sr-only">{t("common.break")}: </span>
      <span className="tabular-nums">{label}</span>
    </span>
  );
}

// ── Type flag row ────────────────────────────────────────────────────

/** Split / night markers. Text + icon, never colour alone (WCAG 1.4.1). */
function TypeFlags({ metrics, t }: { metrics: ShiftCardMetrics; t: TFunction }) {
  return (
    <>
      {metrics.blockCount > 1 ? (
        <span className="inline-flex items-center gap-1 text-[11px] font-bold text-teal-700 dark:text-teal-400">
          <SplitSquareHorizontal aria-hidden="true" className="shrink-0" size={11} />
          {t("shifts.picker.card.blocks", { count: metrics.blockCount })}
        </span>
      ) : null}

      {metrics.crossesMidnight ? (
        <span className="inline-flex items-center gap-1 text-[11px] font-bold text-indigo-700 dark:text-indigo-400">
          <Moon aria-hidden="true" className="shrink-0" size={11} />
          {t("shifts.picker.filter_night")}
        </span>
      ) : null}
    </>
  );
}

// ── Header row ───────────────────────────────────────────────────────

interface HeaderRowProps {
  shift: Shift;
  t: TFunction;
  usageCount?: number;
  workLabel: string;
}

function HeaderRow({ shift, t, usageCount, workLabel }: HeaderRowProps) {
  // A single past selection is noise; the ★ badge earns its place from 2 up.
  const isFrequent = (usageCount ?? 0) > 1;
  const usedLabel = t("shifts.picker.card.used_times", { count: usageCount ?? 0 });

  return (
    <div className="flex items-start justify-between gap-3">
      <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
        <span className="truncate text-sm font-bold tracking-tight text-slate-800 uppercase dark:text-slate-100">
          {shift.name}
        </span>
        {shift.report_symbol ? (
          <span className="rounded-md bg-slate-100 px-1.5 py-0.5 font-mono text-[10px] font-bold text-slate-500 dark:bg-slate-700/60 dark:text-slate-300">
            {shift.report_symbol}
          </span>
        ) : null}
        {isFrequent ? (
          <span
            className="inline-flex items-center gap-0.5 rounded-md bg-amber-100 px-1.5 py-0.5 text-[10px] font-bold text-amber-800 dark:bg-amber-950/60 dark:text-amber-300"
            title={usedLabel}
          >
            <Star aria-hidden="true" className="fill-current" size={9} />
            <span className="tabular-nums">{usageCount}</span>
            <span className="sr-only">{usedLabel}</span>
          </span>
        ) : null}
      </div>

      <span className="shrink-0 text-right">
        <span className="block text-sm font-bold tabular-nums text-primary-700 dark:text-primary-400">
          {workLabel}
        </span>
        <span className="block text-[9px] font-bold tracking-wider text-slate-400 uppercase dark:text-slate-500">
          {t("shifts.picker.card.work_hours")}
        </span>
      </span>
    </div>
  );
}

// ── Props ────────────────────────────────────────────────────────────

export interface ShiftPickerCardProps {
  badge: ShiftBadgeStyle;
  formatTime: (v: string) => string;
  /** True while this card is the active option of the arrow-key roving cursor. */
  isActive: boolean;
  metrics: ShiftCardMetrics;
  onClick: () => void;
  onMouseEnter: () => void;
  shift: Shift;
  t: TFunction;
  /** Selection count from localStorage; renders a ★ badge when > 1. */
  usageCount?: number;
}

// ── Card ─────────────────────────────────────────────────────────────

/**
 * A single selectable shift in the picker. Renders aggregate metrics across all
 * intervals plus a 24h mini timeline, and carries a full text label for screen
 * readers so none of the meaning lives in colour alone (WCAG 1.4.1).
 */
export const ShiftPickerCard = forwardRef<HTMLButtonElement, ShiftPickerCardProps>(
  function ShiftPickerCard(
    { badge, formatTime, isActive, metrics, onClick, onMouseEnter, shift, t, usageCount },
    ref,
  ) {
    const workLabel = formatDurationShort(metrics.totalWorkMinutes);

    const times = metrics.intervals
      .map((iv) => `${formatTime(iv.startTime)}–${formatTime(iv.endTime)}`)
      .join(", ");
    const ariaLabel = [
      shift.name,
      shift.report_symbol ? `(${shift.report_symbol})` : "",
      times,
      `${t("shifts.picker.card.work_hours")}: ${workLabel}`,
      badge.label,
    ]
      .filter(Boolean)
      .join(". ");

    return (
      <Button
        aria-label={ariaLabel}
        className={cn(
          "!h-auto !min-h-0 !p-4 !pl-5 group relative w-full overflow-hidden rounded-3xl border-2 text-left",
          "transition-all duration-200 active:scale-[0.99]",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2",
          "dark:focus-visible:ring-offset-slate-900",
          isActive
            ? "border-primary-500 bg-primary-50/70 shadow-md dark:border-primary-500 dark:bg-primary-950/40"
            : "border-slate-150 bg-white hover:border-primary-300 hover:bg-primary-50/40 dark:border-slate-700/70 dark:bg-slate-800/40 dark:hover:border-primary-700 dark:hover:bg-primary-950/20",
        )}
        data-active={isActive ? "true" : undefined}
        onClick={onClick}
        onMouseEnter={onMouseEnter}
        ref={ref}
        type="button"
        size="sm"
        variant="ghost"
      >
        {/* Accent strip — decorative, meaning is duplicated in the badge text */}
        <span
          aria-hidden="true"
          className={cn("absolute top-0 bottom-0 left-0 w-1.5", badge.accent)}
        />

        <div className="space-y-2.5">
          {/* Row 1: name + symbol + usage, paid hours */}
          <HeaderRow shift={shift} t={t} usageCount={usageCount} workLabel={workLabel} />

          {/* Row 2: times, break, type flags */}
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <TimeRow formatTime={formatTime} metrics={metrics} t={t} />
            <BreakRow formatTime={formatTime} metrics={metrics} t={t} />
            <TypeFlags metrics={metrics} t={t} />
          </div>

          {/* Row 3: 24h mini timeline */}
          <ShiftCardTimeline accentClassName={badge.accent} segments={metrics.segments} />
        </div>
      </Button>
    );
  },
);
