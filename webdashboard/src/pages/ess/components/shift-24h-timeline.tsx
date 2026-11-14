import { Clock } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

interface ShiftTimeWindow {
  breakEndHour?: number; // e.g. 13.0
  breakStartHour?: number; // e.g. 12.0
  endHour: number; // e.g. 17.5 for 17:30
  label?: string;
  startHour: number; // e.g. 8.5 for 08:30
}

interface ActualPunchSpan {
  punchInTime: string; // ISO date string
  punchOutTime?: string | null;
}

interface Shift24hTimelineProps {
  actualPunches?: ActualPunchSpan[];
  className?: string;
  scheduledShift?: ShiftTimeWindow;
  timezone?: string;
}

function timeStrToMinutes(timeIso: string): number {
  const d = new Date(timeIso);
  if (isNaN(d.getTime())) return 0;
  return d.getHours() * 60 + d.getMinutes();
}

function minsToPercent(mins: number): number {
  return Math.min(100, Math.max(0, (mins / 1440) * 100));
}

function formatMinsToTime(mins: number): string {
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return `${h < 10 ? `0${h}` : h}:${m < 10 ? `0${m}` : m}`;
}

export function Shift24hTimeline({
  scheduledShift = {
    startHour: 8.5, // 08:30
    endHour: 17.5, // 17:30
    breakStartHour: 12.0, // 12:00
    breakEndHour: 13.0, // 13:00
    label: "Ca hành chính",
  },
  actualPunches = [],
  className,
}: Shift24hTimelineProps) {
  const { t } = useTranslation();

  // Current real-world minutes
  const [currentMinutes, setCurrentMinutes] = useState(() => {
    const d = new Date();
    return d.getHours() * 60 + d.getMinutes();
  });

  useEffect(() => {
    const timer = setInterval(() => {
      const d = new Date();
      setCurrentMinutes(d.getHours() * 60 + d.getMinutes());
    }, 30000); // Update every 30s
    return () => { clearInterval(timer); };
  }, []);

  // Scheduled shift boundaries
  const shiftStartMin = scheduledShift.startHour * 60;
  const shiftEndMin = scheduledShift.endHour * 60;
  const shiftLeft = minsToPercent(shiftStartMin);
  const shiftWidth = minsToPercent(shiftEndMin - shiftStartMin);

  // Break window boundaries
  const breakStartMin = (scheduledShift.breakStartHour ?? 12) * 60;
  const breakEndMin = (scheduledShift.breakEndHour ?? 13) * 60;
  const breakLeft = minsToPercent(breakStartMin);
  const breakWidth = minsToPercent(breakEndMin - breakStartMin);

  const nowLeft = minsToPercent(currentMinutes);

  const hourTicks = [0, 4, 8, 12, 16, 20, 24];

  return (
    <div
      aria-label={t("ess.clock.timeline_label", "Trực quan hóa ca làm việc 24h")}
      className={cn(
        "p-5 rounded-3xl bg-white dark:bg-slate-900 border border-slate-200/80 dark:border-slate-800 shadow-sm space-y-4",
        className
      )}
      data-testid="shift-24h-timeline"
    >
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
        <div className="space-y-0.5">
          <h4 className="text-sm font-bold text-slate-900 dark:text-slate-100 flex items-center gap-2">
            <Clock className="h-4 w-4 text-primary-600 dark:text-primary-400" />
            {t("ess.clock.timeline_title", "Dòng thời gian ca làm việc 24h")}
          </h4>
          <p className="text-xs text-slate-500 dark:text-slate-400">
            {scheduledShift.label}: {formatMinsToTime(shiftStartMin)} →{" "}
            {formatMinsToTime(shiftEndMin)} (Nghỉ trưa:{" "}
            {formatMinsToTime(breakStartMin)} - {formatMinsToTime(breakEndMin)})
          </p>
        </div>

        <div className="flex items-center gap-2 text-xs font-semibold text-slate-600 dark:text-slate-300">
          <span className="flex h-2 w-2 rounded-full bg-rose-500 animate-pulse" />
          <span>
            {t("ess.clock.now_time", "Hiện tại:")}{" "}
            <span className="font-mono font-bold text-slate-900 dark:text-slate-100">
              {formatMinsToTime(currentMinutes)}
            </span>
          </span>
        </div>
      </div>

      {/* ── 24h Visual Track Container ───────────────────────────────── */}
      <div className="pt-6 pb-4">
        <div className="relative w-full h-10 rounded-2xl bg-slate-100 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 overflow-visible">
          {/* 1. Scheduled Shift Box */}
          <div
            className="absolute top-0 bottom-0 bg-primary-100/70 dark:bg-primary-950/50 border-x-2 border-primary-400/80 dark:border-primary-500/80 rounded-lg flex items-center justify-start pl-3 transition-all"
            data-testid="scheduled-shift-box"
            style={{ left: `${shiftLeft}%`, width: `${shiftWidth}%` }}
          >
            <span className="text-[10px] font-bold text-primary-800 dark:text-primary-300 uppercase tracking-tighter hidden md:inline">
              {scheduledShift.label}
            </span>
          </div>

          {/* 2. Break Window Box */}
          <div
            className="absolute top-1 bottom-1 bg-amber-200/60 dark:bg-amber-900/40 border border-dashed border-amber-400/80 dark:border-amber-600/80 rounded-md flex items-center justify-center"
            data-testid="break-window-box"
            style={{ left: `${breakLeft}%`, width: `${breakWidth}%` }}
            title={t("ess.clock.break_tooltip", "Giờ nghỉ trưa")}
          >
            <span className="text-[9px] font-bold text-amber-900 dark:text-amber-300 uppercase tracking-tighter hidden sm:inline">
              Nghỉ
            </span>
          </div>

          {/* 3. Actual Worked Spans */}
          {actualPunches.map((punch, idx) => {
            const inMins = timeStrToMinutes(punch.punchInTime);
            const outMins = punch.punchOutTime
              ? timeStrToMinutes(punch.punchOutTime)
              : currentMinutes;

            const punchLeft = minsToPercent(inMins);
            const punchWidth = Math.max(1, minsToPercent(outMins - inMins));

            return (
              <React.Fragment key={idx}>
                {/* Worked bar */}
                <div
                  className="absolute top-2 bottom-2 bg-gradient-to-r from-emerald-500 to-teal-500 rounded-full shadow-sm z-10 transition-all opacity-90"
                  data-testid={`worked-bar-${idx}`}
                  style={{ left: `${punchLeft}%`, width: `${punchWidth}%` }}
                />

                {/* Punch In Marker Dot */}
                <div
                  className="absolute -top-2.5 z-20 -translate-x-1/2 flex flex-col items-center group cursor-pointer"
                  data-testid={`punch-in-dot-${idx}`}
                  style={{ left: `${punchLeft}%` }}
                >
                  <span className="h-4 w-4 rounded-full bg-emerald-500 border-2 border-white dark:border-slate-900 shadow-md flex items-center justify-center text-[8px] font-bold text-white">
                    ✓
                  </span>
                  <span className="text-[9px] font-mono font-bold text-emerald-700 dark:text-emerald-300 mt-0.5 bg-white/90 dark:bg-slate-900/90 px-1 rounded shadow-xs">
                    {formatMinsToTime(inMins)}
                  </span>
                </div>

                {/* Punch Out Marker Dot if checked out */}
                {!!punch.punchOutTime && (
                  <div
                    className="absolute -top-2.5 z-20 -translate-x-1/2 flex flex-col items-center group cursor-pointer"
                    data-testid={`punch-out-dot-${idx}`}
                    style={{ left: `${minsToPercent(outMins)}%` }}
                  >
                    <span className="h-4 w-4 rounded-full bg-teal-600 border-2 border-white dark:border-slate-900 shadow-md flex items-center justify-center text-[8px] font-bold text-white">
                      ✕
                    </span>
                    <span className="text-[9px] font-mono font-bold text-teal-700 dark:text-teal-300 mt-0.5 bg-white/90 dark:bg-slate-900/90 px-1 rounded shadow-xs">
                      {formatMinsToTime(outMins)}
                    </span>
                  </div>
                )}
              </React.Fragment>
            );
          })}

          {/* 4. Current Time Scrubber Needle */}
          <div
            className="absolute -top-3 -bottom-3 w-0.5 bg-rose-500 z-30 -translate-x-1/2 flex flex-col items-center"
            data-testid="now-scrubber-needle"
            style={{ left: `${nowLeft}%` }}
          >
            <span className="h-2.5 w-2.5 rounded-full bg-rose-500 ring-4 ring-rose-500/20 shadow-md" />
          </div>
        </div>

        {/* 24h Axis Hour Ticks */}
        <div
          className="relative w-full h-5 mt-2 text-[10px] font-mono text-slate-400 dark:text-slate-500 select-none"
          data-testid="timeline-hour-axis"
        >
          {hourTicks.map((h) => {
            const left = minsToPercent(h * 60);
            return (
              <span
                className="absolute -translate-x-1/2"
                key={h}
                style={{ left: `${left}%` }}
              >
                {h < 10 ? `0${h}:00` : `${h}:00`}
              </span>
            );
          })}
        </div>
      </div>

      {/* ── Legend ───────────────────────────────────────────────────── */}
      <div className="flex flex-wrap items-center justify-center sm:justify-start gap-4 pt-1 text-xs text-slate-600 dark:text-slate-400 border-t border-slate-100 dark:border-slate-800">
        <div className="flex items-center gap-1.5">
          <span className="h-3 w-3 rounded bg-primary-200 dark:bg-primary-900 border border-primary-400" />
          <span>{t("ess.clock.legend_shift", "Ca quy định")}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="h-3 w-3 rounded bg-amber-200 dark:bg-amber-800 border border-dashed border-amber-400" />
          <span>{t("ess.clock.legend_break", "Nghỉ giữa ca")}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="h-3 w-3 rounded-full bg-emerald-500" />
          <span>{t("ess.clock.legend_punch", "Giờ làm thực tế")}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="h-3 w-1 bg-rose-500 rounded-full" />
          <span>{t("ess.clock.legend_now", "Thời điểm hiện tại")}</span>
        </div>
      </div>
    </div>
  );
}
