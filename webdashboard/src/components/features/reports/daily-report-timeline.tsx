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

function getMinutesFromDayStart(isoString: string, baseDateStr: string) {
  const d = new Date(isoString);
  const [yr, mo, dy] = baseDateStr.split("-").map(Number);
  const baseDate = new Date(yr, mo - 1, dy);
  const localD = new Date(d.getFullYear(), d.getMonth(), d.getDate(), d.getHours(), d.getMinutes(), d.getSeconds());
  return Math.round((localD.getTime() - baseDate.getTime()) / 60000);
}

function parseTimeToMinutes(timeStr: string | null | undefined) {
  if (!timeStr) return null;
  const [h, m] = timeStr.split(":").map(Number);
  return h * 60 + m;
}

function formatMinutesToTime(totalMins: number) {
  const mins = totalMins % 1440;
  return `${Math.floor(mins / 60).toString().padStart(2, "0")}:${(mins % 60).toString().padStart(2, "0")}`;
}

// eslint-disable-next-line complexity -- overnight/break boundary calculation = 15
function useTimelineBounds(row: DailyAttendanceReport) {
  const startMin = parseTimeToMinutes(row.scheduled_start);
  let endMin = parseTimeToMinutes(row.scheduled_end);
  let bStart = parseTimeToMinutes(row.break_start);
  let bEnd = parseTimeToMinutes(row.break_end);

  if (startMin !== null && endMin !== null) {
    if (endMin < startMin) endMin += 1440;
    if (bStart !== null && bStart < startMin) bStart += 1440;
    if (bEnd !== null && bEnd! < bStart!) bEnd += 1440;
  }

  const metadata = row.generation_metadata as { time_entries?: TimeEntry[] } | undefined;
  const rawEntries = metadata?.time_entries ?? [];
  const timeEntries = rawEntries.map((e) => {
    const inMin = getMinutesFromDayStart(e.punch_in_time, row.date);
    const outMin = e.punch_out_time ? getMinutesFromDayStart(e.punch_out_time, row.date) : null;
    return { ...e, inMin, outMin };
  }).sort((a, b) => a.inMin - b.inMin);

  const workEntries = timeEntries.filter((e) => e.entry_type === "work");

  let timelineStart = startMin !== null ? startMin - 120 : 480;
  let timelineEnd = endMin !== null ? endMin + 120 : 1020;
  if (timeEntries.length > 0) {
    const earliestPunch = timeEntries[0].inMin;
    const latestPunch = Math.max(...timeEntries.map((e) => e.outMin ?? e.inMin + 60));
    if (earliestPunch < timelineStart) timelineStart = earliestPunch - 30;
    if (latestPunch > timelineEnd) timelineEnd = latestPunch + 30;
  }

  const totalRange = timelineEnd - timelineStart;
  const getPercent = (mins: number) => Math.max(0, Math.min(100, ((mins - timelineStart) / totalRange) * 100));

  return { startMin, endMin, bStart, bEnd, timeEntries, workEntries, getPercent };
}

function ScheduleTick({ label, pct, color }: { label: string; pct: number; color: string }) {
  return (
    <div className="absolute flex flex-col items-center -translate-x-1/2 bottom-0" style={{ left: `${pct}%` }}>
      <span className={`${color} mb-1`}>{label}</span>
      <div className={`w-px h-1.5 ${color.replace("text-", "bg-")}`} />
    </div>
  );
}

function ActualTick({ label, pct }: { label: string; pct: number }) {
  return (
    <div className="absolute flex flex-col items-center -translate-x-1/2 top-0" style={{ left: `${pct}%` }}>
      <div className="w-px h-1.5 bg-emerald-400 mb-1" />
      <span className="text-emerald-600 font-bold">{label}</span>
    </div>
  );
}

function TimelineSession({ entry, formatTime, t, getPercent }: {
  entry: ReturnType<typeof useTimelineBounds>["timeEntries"][number];
  formatTime: (t: string | null) => string;
  t: (k: string) => string;
  getPercent: (m: number) => number;
}) {
  const startPct = getPercent(entry.inMin);
  const endPct = getPercent(entry.outMin ?? (entry.inMin + 30));
  const isBreak = entry.entry_type === "break";
  const isBreakOt = entry.is_break_overtime;

  return (
    <div
      className={cn(
        "absolute h-7 rounded-lg group/session transition-all hover:scale-[1.02] shadow-sm flex items-center justify-between px-2 cursor-help text-[9px] font-bold text-white border overflow-hidden",
        isBreak ? "bg-slate-400 border-slate-500/20" : isBreakOt ? "bg-amber-500 border-amber-600/20" : "bg-emerald-500/90 border-emerald-600/20",
      )}
      key={entry.id}
      style={{ left: `${startPct}%`, width: `${Math.max(1, endPct - startPct)}%` }}
    >
      <span className="truncate max-w-full">{isBreak ? t("common.break") : t("reports.quick_stats.std_hours")}</span>
      <div className="absolute hidden group-hover/session:flex flex-col bg-slate-950/95 backdrop-blur-md text-white p-2.5 rounded-xl border border-white/10 shadow-premium pointer-events-none z-50 text-[10px] w-48 font-normal leading-normal bottom-full mb-2 left-1/2 -translate-x-1/2">
        <div className="font-bold border-b border-white/10 pb-1 mb-1 flex items-center justify-between">
          <span>{isBreak ? t("common.break") : t("reports.quick_stats.std_hours")}</span>
          {entry.duration_minutes !== null && <span className="font-mono text-primary-400">{Math.floor(entry.duration_minutes / 60)}h{entry.duration_minutes % 60}m</span>}
        </div>
        <div className="flex justify-between text-white/60"><span>{t("common.in")}:</span><span className="font-mono">{formatTime(entry.punch_in_time)}</span></div>
        <div className="flex justify-between text-white/60"><span>{t("common.out")}:</span><span className="font-mono">{entry.punch_out_time ? formatTime(entry.punch_out_time) : t("reports.confirm.missing_out")}</span></div>
        {!!entry.note && <div className="text-[9px] text-orange-300 italic border-t border-white/5 pt-1 mt-1 truncate">Note: {entry.note}</div>}
      </div>
    </div>
  );
}

export function DailyReportTimeline({ row, formatTime, t }: { formatTime: (t: string | null) => string; row: DailyAttendanceReport; t: (k: string) => string }) {
  const { startMin, endMin, bStart, bEnd, timeEntries, workEntries, getPercent } = useTimelineBounds(row);
  const isMultiSegment = (row.segment_count ?? 1) > 1;
  const hasSegmentData = !!row.segment_type;

  return (
    <div className="w-full space-y-2 select-none py-2 px-1">
      {hasSegmentData && (
        <div className="flex items-center gap-2 mb-2">
          <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{row.interval_name ?? row.segment_type}</span>
          {isMultiSegment && <span className="text-[9px] font-bold text-slate-400 bg-slate-100 px-2 py-0.5 rounded-full">{(row.sort_order ?? 0) + 1} / {row.segment_count}</span>}
        </div>
      )}
      <div className="relative h-6 w-full text-[9px] font-bold font-mono text-slate-400">
        {startMin !== null && <ScheduleTick label={formatMinutesToTime(startMin)} pct={getPercent(startMin)} color="text-blue-500" />}
        {endMin !== null && <ScheduleTick label={formatMinutesToTime(endMin)} pct={getPercent(endMin)} color="text-blue-500" />}
      </div>

      <div className="relative h-9 w-full bg-slate-100 rounded-xl border border-slate-200/50 shadow-inner flex items-center">
        {startMin !== null && endMin !== null && (
          <div className="absolute h-full border-l border-r border-dashed border-blue-400/50 bg-blue-50/15 flex items-center justify-center pointer-events-none"
            style={{ left: `${getPercent(startMin)}%`, width: `${getPercent(endMin) - getPercent(startMin)}%` }}>
            {bStart !== null && bEnd !== null && (
              <div className="absolute h-full bg-slate-200/40 border-l border-r border-dotted border-slate-400/30"
                style={{ left: `${((bStart - startMin) / (endMin - startMin)) * 100}%`, width: `${((bEnd - bStart) / (endMin - startMin)) * 100}%` }} />
            )}
          </div>
        )}
        {timeEntries.map((entry) => (
          <TimelineSession key={entry.id} entry={entry} formatTime={formatTime} t={t} getPercent={getPercent} />
        ))}
      </div>

      <div className="relative h-6 w-full text-[9px] font-bold font-mono text-slate-400">
        {workEntries.map((e) => (
          <React.Fragment key={`tick-${e.id}`}>
            <ActualTick label={formatMinutesToTime(e.inMin)} pct={getPercent(e.inMin)} />
            {e.outMin !== null && <ActualTick label={formatMinutesToTime(e.outMin)} pct={getPercent(e.outMin)} />}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}
