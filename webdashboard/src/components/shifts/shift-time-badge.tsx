import { Clock, Info } from "lucide-react";
import React from "react";

import type { ShiftWithIntervals } from "@/pages/shifts/hooks/use-shifts-types";

import { Shift } from "@/lib/api";

// ── ShiftTimeRange ─────────────────────────────────────────────────────

function ShiftTimeRange({ startTime, endTime, formatTime }: {
  startTime: string;
  endTime: string;
  formatTime: (v: string) => string;
}) {
  return (
    <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
      <Clock size={10} /> {startTime ? formatTime(startTime) : "—"} - {endTime ? formatTime(endTime) : "—"}
    </div>
  );
}

// ── ShiftBreakInfo ─────────────────────────────────────────────────────

function ShiftBreakInfo({ breakStart, breakEnd, breakDurationMinutes, t }: {
  breakStart?: string;
  breakEnd?: string;
  breakDurationMinutes: number;
  t: (key: string) => string;
}) {
  return (
    <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
      <Info size={10} />
      {breakStart ? (
        <span>{breakStart.substring(0, 5)}-{breakEnd?.substring(0, 5)}</span>
      ) : (
        <span>{t("common.break")} {breakDurationMinutes}m</span>
      )}
    </div>
  );
}

// ── ShiftTimeBadge ─────────────────────────────────────────────────────

interface ShiftTimeBadgeProps {
  formatTime: (v: string) => string;
  shift: Shift;
  t: (key: string) => string;
}

export function ShiftTimeBadge({ shift, formatTime, t }: ShiftTimeBadgeProps) {
  const typed = shift as ShiftWithIntervals;
  const iv = typed.intervals?.[0];
  const st = iv?.start_time ?? "";
  const et = iv?.end_time ?? "";

  return (
    <>
      <ShiftTimeRange endTime={et} formatTime={formatTime} startTime={st} />
      <ShiftBreakInfo
        breakDurationMinutes={iv?.break_duration_minutes ?? 0}
        breakEnd={iv?.break_end}
        breakStart={iv?.break_start}
        t={t}
      />
    </>
  );
}
