import { ChevronDown, ChevronUp } from "lucide-react";
import React, { useState } from "react";

import type { ShiftVersionHistoryItem } from "@/lib/api/types";

import Button from "@/components/ui/button";

// ── Types ──────────────────────────────────────────────────────────

export interface IntervalDiffRowProps {
  currentInterval: ShiftVersionHistoryItem["intervals"][number] | undefined;
  idx: number;
  previousInterval: ShiftVersionHistoryItem["intervals"][number] | undefined;
  t: (key: string, options?: Record<string, unknown>) => string;
}

interface IntervalChangeSet {
  breakChanged: boolean;
  breakCurr: number;
  breakPrev: number;
  endChanged: boolean;
  endCurr: string;
  endPrev: string;
  startChanged: boolean;
  startCurr: string;
  startPrev: string;
}

interface IntervalDiffProps {
  current: ShiftVersionHistoryItem;
  previous: ShiftVersionHistoryItem | null;
  t: (key: string, options?: Record<string, unknown>) => string;
}

// ── Pure helpers ────────────────────────────────────────────────────

/** Empty interval sentinel — provides fallback values for optional fields. */
const NULL_INTERVAL = {
  start_time: "" as string | null | undefined,
  end_time: "" as string | null | undefined,
  break_duration_minutes: 0 as number | null | undefined,
};

/** Extract display-safe string/number values from a possibly-undefined interval. */
function extractIntervalFields(iv: ShiftVersionHistoryItem["intervals"][number] | undefined) {
  const i: typeof NULL_INTERVAL = (iv as typeof NULL_INTERVAL | undefined) ?? NULL_INTERVAL;
  return {
    startTime: (i.start_time ?? "").substring(0, 5) || "",
    endTime: (i.end_time ?? "").substring(0, 5) || "",
    breakMinutes: i.break_duration_minutes ?? 0,
  };
}

/**
 * Compute all interval field diffs between current and previous version.
 */
function computeIntervalChanges(
  ci: ShiftVersionHistoryItem["intervals"][number] | undefined,
  pi: ShiftVersionHistoryItem["intervals"][number] | undefined,
): IntervalChangeSet | null {
  if (!ci && !pi) return null;
  const c = extractIntervalFields(ci);
  const p = extractIntervalFields(pi);
  return {
    breakChanged: c.breakMinutes !== p.breakMinutes,
    breakCurr: c.breakMinutes,
    breakPrev: p.breakMinutes,
    endChanged: c.endTime !== p.endTime,
    endCurr: c.endTime,
    endPrev: p.endTime,
    startChanged: c.startTime !== p.startTime,
    startCurr: c.startTime,
    startPrev: p.startTime,
  };
}

/** Returns true when no fields in the change set differ. */
function hasNoChanges(ch: IntervalChangeSet): boolean {
  return !ch.startChanged && !ch.endChanged && !ch.breakChanged;
}

// ── Sub-components ─────────────────────────────────────────────────

/** Renders a single time field diff (label + old→new comparison). */
function TimeFieldDiff({
  label,
  changed,
  prevValue,
  currValue,
}: {
  label: string;
  changed: boolean;
  prevValue?: string;
  currValue?: string;
}) {
  return (
    <>
      <span className="font-bold text-slate-600 dark:text-slate-300">{label}:</span>
      {prevValue ? <span className="text-slate-400 line-through">{prevValue.substring(0, 5)}</span> : null}
      {currValue ? (
        <span className={changed ? "text-amber-600 dark:text-amber-400 font-bold" : "text-slate-600 dark:text-slate-300"}>
          {currValue.substring(0, 5)}
        </span>
      ) : null}
    </>
  );
}

/** Renders break duration diff (old→new). */
function BreakFieldDiff({
  t,
  prevMinutes,
  currMinutes,
}: {
  t: (key: string, options?: Record<string, unknown>) => string;
  prevMinutes?: number;
  currMinutes?: number;
}) {
  return (
    <div className="flex items-center gap-2">
      <span className="font-bold text-slate-600 dark:text-slate-300">
        {t("shifts.table.break_time")}:
      </span>
      <span className="text-slate-400 line-through">{prevMinutes}{t("common.minutes")}</span>
      <span className="text-amber-600 dark:text-amber-400 font-bold">{currMinutes}{t("common.minutes")}</span>
    </div>
  );
}

/** Renders time range changes (start + end time diff in one row). */
function TimeRangeChanged({
  ch,
  t,
}: {
  ch: IntervalChangeSet;
  t: (key: string, options?: Record<string, unknown>) => string;
}) {
  if (!ch.startChanged && !ch.endChanged) return null;
  return (
    <div className="flex items-center gap-2 flex-wrap">
      <TimeFieldDiff changed={ch.startChanged} currValue={ch.startCurr} label={t("shifts.modal.start_time")} prevValue={ch.startPrev} />
      <span className="text-slate-300 dark:text-slate-600 mx-1">|</span>
      <TimeFieldDiff changed={ch.endChanged} currValue={ch.endCurr} label={t("shifts.modal.end_time")} prevValue={ch.endPrev} />
    </div>
  );
}

/** Renders a single interval comparison row (start/end time + break changes). */
function IntervalDiffRow({ currentInterval: ci, previousInterval: pi, idx, t }: IntervalDiffRowProps) {
  const ch = computeIntervalChanges(ci, pi);
  if (!ch) return null;
  if (hasNoChanges(ch)) {
    return (
      <div className="text-[10px] text-slate-400 dark:text-slate-500 font-medium" key={idx}>
        {t("shifts.revisions.no_changes")}
      </div>
    );
  }

  return (
    <div className="text-[11px] font-mono space-y-1" key={idx}>
      <TimeRangeChanged ch={ch} t={t} />
      {ch.breakChanged ? <BreakFieldDiff currMinutes={ch.breakCurr} prevMinutes={ch.breakPrev} t={t} /> : null}
    </div>
  );
}

// ── Interval Diff Section ──────────────────────────────────────────

export function IntervalDiff({ current, previous, t }: IntervalDiffProps) {
  const [expanded, setExpanded] = useState(false);

  if (!previous?.intervals.length) {
    return null;
  }

  const currIntervals = current.intervals;
  const prevIntervals = previous.intervals;
  const maxLen = Math.max(currIntervals.length, prevIntervals.length);

  return (
    <div className="mt-3">
      <Button
        className="flex items-center gap-1.5 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest hover:text-primary-500 dark:hover:text-primary-400 transition-colors"
        data-testid="shift-diff-toggle"
        onClick={() => { setExpanded((v) => !v); }}
        type="button"
        variant="ghost"
      >
        {expanded
          ? <ChevronUp className="w-3 h-3" />
          : <ChevronDown className="w-3 h-3" />}
        {expanded
          ? t("shifts.revisions.hide_diff")
          : t("shifts.revisions.show_diff")}
      </Button>

      {expanded ? (
        <div className="mt-2 space-y-2 pl-4 border-l-2 border-amber-200 dark:border-amber-800">
          {Array.from({ length: maxLen }).map((_, idx) => (
            <IntervalDiffRow
              currentInterval={currIntervals[idx]}
              idx={idx}
              key={idx}
              previousInterval={prevIntervals[idx]}
              t={t}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}
