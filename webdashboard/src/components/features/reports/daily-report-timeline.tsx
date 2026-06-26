import React from "react";

import { DailyAttendanceReport } from "@/lib/api";
import { cn } from "@/lib/utils";

export interface TimeEntry {
  duration_minutes: number | null;
  entry_type: "work" | "break";
  id: string;
  is_break_overtime?: boolean;
  note: string | null;
  punch_in_time: string;
  punch_out_time: string | null;
}

interface DailyReportTimelineProps {
  formatTime: (time: string | null) => string;
  row: DailyAttendanceReport;
  t: (key: string) => string;
}

// Convert ISO datetime string to local minutes from midnight relative to a base date.
// Handles timezone offsets and overnight transitions.
function getMinutesFromDayStart(isoString: string, baseDateStr: string) {
  const d = new Date(isoString);
  const [yr, mo, dy] = baseDateStr.split("-").map(Number);
  const baseDate = new Date(yr, mo - 1, dy); // base local midnight
  
  const localD = new Date(
    d.getFullYear(),
    d.getMonth(),
    d.getDate(),
    d.getHours(),
    d.getMinutes(),
    d.getSeconds()
  );
  
  const diffMs = localD.getTime() - baseDate.getTime();
  return Math.round(diffMs / 60000);
}

function parseTimeToMinutes(timeStr: string | null | undefined) {
  if (!timeStr) return null;
  const [h, m] = timeStr.split(":").map(Number);
  return h * 60 + m;
}

function formatMinutesToTime(totalMins: number) {
  const mins = totalMins % 1440;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`;
}

export function DailyReportTimeline({ row, formatTime, t }: DailyReportTimelineProps) {
  // 1. Parse shift config boundaries
  const startMin = parseTimeToMinutes(row.scheduled_start);
  let endMin = parseTimeToMinutes(row.scheduled_end);
  let bStart = parseTimeToMinutes(row.break_start);
  let bEnd = parseTimeToMinutes(row.break_end);

  if (startMin !== null && endMin !== null) {
    if (endMin < startMin) {
      endMin += 1440;
    }
    if (bStart !== null && bStart < startMin) {
      bStart += 1440;
    }
    if (bEnd !== null && bEnd < bStart!) {
      bEnd += 1440;
    }
  }

  // 2. Parse time entries
  const metadata = row.generation_metadata as { time_entries?: TimeEntry[] } | undefined;
  const rawEntries = metadata?.time_entries ?? [];
  
  const timeEntries = rawEntries.map((e) => {
    const inMin = getMinutesFromDayStart(e.punch_in_time, row.date);
    const outMin = e.punch_out_time
      ? getMinutesFromDayStart(e.punch_out_time, row.date)
      : null;
    return {
      ...e,
      inMin,
      outMin,
    };
  });

  // Sort chronological
  timeEntries.sort((a, b) => a.inMin - b.inMin);

  // Work-only entries (exclude auto-computed breaks) for axis display
  const workEntries = timeEntries.filter((e) => e.entry_type === "work");

  // 3. Determine overall timeline bounds
  let timelineStart = startMin !== null ? startMin - 120 : 480; // Default 08:00 - 2 hours = 06:00
  let timelineEnd = endMin !== null ? endMin + 120 : 1020;    // Default 17:00 + 2 hours = 19:00

  // Expand bounds to fit any punches outside the default/shift window
  if (timeEntries.length > 0) {
    const earliestPunch = timeEntries[0].inMin;
    const latestPunch = Math.max(
      ...timeEntries.map((e) => e.outMin ?? e.inMin + 60)
    );
    if (earliestPunch < timelineStart) {
      timelineStart = earliestPunch - 30;
    }
    if (latestPunch > timelineEnd) {
      timelineEnd = latestPunch + 30;
    }
  }

  const totalRange = timelineEnd - timelineStart;
  const getPercent = (mins: number) => {
    const pct = ((mins - timelineStart) / totalRange) * 100;
    return Math.max(0, Math.min(100, pct));
  };

  const isMultiSegment = (row.segment_count ?? 1) > 1;
  const hasSegmentData = !!row.segment_type;

  return (
    <div className="w-full space-y-2 select-none py-2 px-1">
      {/* Segment label (segment-centric path) */}
      {hasSegmentData ? (
        <div className="flex items-center gap-2 mb-2">
          <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
            {row.interval_name ?? row.segment_type}
          </span>
          {isMultiSegment ? (
            <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-full">
              {(row.sort_order ?? 0) + 1} / {row.segment_count}
            </span>
          ) : null}
        </div>
      ) : null}

      {/* 1. Scheduled Axis (Above Visual Bar) */}
      <div className="relative h-6 w-full text-[9px] font-bold font-mono text-slate-400 dark:text-slate-500">
        {startMin !== null && (
          <div
            className="absolute flex flex-col items-center -translate-x-1/2 bottom-0"
            style={{ left: `${getPercent(startMin)}%` }}
          >
            <span className="text-blue-500 dark:text-blue-400 mb-1">{formatMinutesToTime(startMin)}</span>
            <div className="w-px h-1.5 bg-blue-400 dark:bg-blue-600" />
          </div>
        )}

        {endMin !== null && (
          <div
            className="absolute flex flex-col items-center -translate-x-1/2 bottom-0"
            style={{ left: `${getPercent(endMin)}%` }}
          >
            <span className="text-blue-500 dark:text-blue-400 mb-1">{formatMinutesToTime(endMin)}</span>
            <div className="w-px h-1.5 bg-blue-400 dark:bg-blue-600" />
          </div>
        )}
      </div>

      {/* 2. Visual Bar Container */}
      <div className="relative h-9 w-full bg-slate-100 dark:bg-slate-800 rounded-xl border border-slate-200/50 dark:border-slate-700/50 shadow-inner flex items-center">
        
        {/* 1. Scheduled Shift Area (dashed overlay) */}
        {startMin !== null && endMin !== null && (
          <div
            className="absolute h-full border-l border-r border-dashed border-blue-400/50 dark:border-blue-500/30 bg-blue-50/15 dark:bg-blue-950/5 flex items-center justify-center pointer-events-none"
            style={{
              left: `${getPercent(startMin)}%`,
              width: `${getPercent(endMin) - getPercent(startMin)}%`,
            }}
          >
            {/* Scheduled Break Area */}
            {bStart !== null && bEnd !== null && (
              <div
                className="absolute h-full bg-slate-200/40 dark:bg-slate-700/30 border-l border-r border-dotted border-slate-400/30"
                style={{
                  left: `${((bStart - startMin) / (endMin - startMin)) * 100}%`,
                  width: `${((bEnd - bStart) / (endMin - startMin)) * 100}%`,
                }}
              />
            )}
          </div>
        )}

        {/* 2. Actual Session Blocks */}
        {timeEntries.map((entry) => {
          const startPct = getPercent(entry.inMin);
          const endPct = getPercent(entry.outMin ?? (entry.inMin + 30));
          const widthPct = Math.max(1, endPct - startPct);

          const isBreak = entry.entry_type === "break";
          const isBreakOt = entry.is_break_overtime;

          return (
            <div
              className={cn(
                "absolute h-7 rounded-lg group/session transition-all hover:scale-[1.02] shadow-sm flex items-center justify-between px-2 cursor-help text-[9px] font-bold text-white border overflow-hidden",
                isBreak
                  ? "bg-slate-400 dark:bg-slate-600 border-slate-500/20"
                  : isBreakOt
                  ? "bg-amber-500 dark:bg-amber-600 border-amber-600/20"
                  : "bg-emerald-500/90 dark:bg-emerald-600/90 border-emerald-600/20"
              )}
              key={entry.id}
              style={{
                left: `${startPct}%`,
                width: `${widthPct}%`,
              }}
            >
              <span className="truncate max-w-full">
                {isBreak ? t("common.break") : t("reports.quick_stats.std_hours")}
              </span>
              
              {/* Premium Hover Card details inside HTML title tooltip fallback */}
              <div className="absolute hidden group-hover/session:flex flex-col bg-slate-950/95 dark:bg-slate-900/95 backdrop-blur-md text-white p-2.5 rounded-xl border border-white/10 shadow-premium pointer-events-none z-50 text-[10px] w-48 font-normal leading-normal bottom-full mb-2 left-1/2 -translate-x-1/2">
                <div className="font-bold border-b border-white/10 pb-1 mb-1 flex items-center justify-between">
                  <span>{isBreak ? t("common.break") : t("reports.quick_stats.std_hours")}</span>
                  {entry.duration_minutes !== null && (
                    <span className="font-mono text-primary-400">
                      {Math.floor(entry.duration_minutes / 60)}h{entry.duration_minutes % 60}m
                    </span>
                  )}
                </div>
                <div className="flex justify-between text-white/60">
                  <span>{t("common.in")}:</span>
                  <span className="font-mono">{formatTime(entry.punch_in_time)}</span>
                </div>
                <div className="flex justify-between text-white/60">
                  <span>{t("common.out")}:</span>
                  <span className="font-mono">
                    {entry.punch_out_time ? formatTime(entry.punch_out_time) : t("reports.confirm.missing_out")}
                  </span>
                </div>
                {!!entry.note && (
                  <div className="text-[9px] text-orange-300 italic border-t border-white/5 pt-1 mt-1 truncate">
                    Note: {entry.note}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* 3. Actual Axis (Below Visual Bar) — work entries only (no auto-computed break boundaries) */}
      <div className="relative h-6 w-full text-[9px] font-bold font-mono text-slate-400 dark:text-slate-500">
        {workEntries.map((e) => (
          <React.Fragment key={`tick-${e.id}`}>
            <div
              className="absolute flex flex-col items-center -translate-x-1/2 top-0"
              style={{ left: `${getPercent(e.inMin)}%` }}
            >
              <div className="w-px h-1.5 bg-emerald-400 dark:bg-emerald-600 mb-1" />
              <span className="text-emerald-600 dark:text-emerald-500 font-bold">
                {formatMinutesToTime(e.inMin)}
              </span>
            </div>
            {e.outMin !== null && (
              <div
                className="absolute flex flex-col items-center -translate-x-1/2 top-0"
                style={{ left: `${getPercent(e.outMin)}%` }}
              >
                <div className="w-px h-1.5 bg-emerald-400 dark:bg-emerald-600 mb-1" />
                <span className="text-emerald-600 dark:text-emerald-500 font-bold">
                  {formatMinutesToTime(e.outMin)}
                </span>
              </div>
            )}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}
