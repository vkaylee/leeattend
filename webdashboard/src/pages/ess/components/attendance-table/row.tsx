import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { AttendanceDailySummary, AttendanceEntry } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { MetadataPopover } from "./metadata-popover";
import { OtStatusCell } from "./ot-status-cell";

interface AttendanceRowProps {
  entry: AttendanceEntry;
  formatDate: (d: string) => string;
  formatTime: (d: string) => string;
  getSummary: (date: string) => AttendanceDailySummary | undefined;
  setClaimOpen: (open: boolean) => void;
  setSelectedSummary: (summary: AttendanceDailySummary) => void;
}

export function AttendanceRow({
  entry,
  formatDate,
  formatTime,
  getSummary,
  setClaimOpen,
  setSelectedSummary,
}: AttendanceRowProps) {
  const { t } = useTranslation();
  const dateString = format(new Date(entry.punch_in_time), "yyyy-MM-dd");
  const summary = getSummary(dateString);

  return (
    <tr
      className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group"
      key={entry.id}
    >
      <td className="px-10 py-6">
        <div className="flex items-center gap-6">
          <div className="w-14 h-14 rounded-2xl bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 flex items-center justify-center font-bold text-2xl shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 transition-transform">
            {format(new Date(entry.punch_in_time), "dd")}
          </div>
          <div>
            <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1.5">
              {format(new Date(entry.punch_in_time), "EEEE")}
            </div>
            <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] opacity-70 leading-none">
              {formatDate(entry.punch_in_time)}
            </div>
          </div>
        </div>
      </td>
      <td className="px-8 py-6 text-center">
        <div className="flex flex-col items-center gap-2">
          <span className="text-sm font-bold font-mono text-slate-700 dark:text-slate-200 tracking-tighter">
            {formatTime(entry.punch_in_time)}
          </span>
          <div className="w-1.5 h-1.5 bg-emerald-50 dark:bg-emerald-950 rounded-full shadow-sm shadow-emerald-500/50 animate-pulse"></div>
        </div>
      </td>
      <td className="px-8 py-6 text-center">
        {entry.punch_out_time ? (
          <div className="flex flex-col items-center gap-2">
            <span className="text-sm font-bold font-mono text-slate-700 dark:text-slate-200 tracking-tighter">
              {formatTime(entry.punch_out_time)}
            </span>
            <div className="w-1.5 h-1.5 bg-rose-50 dark:bg-rose-950 rounded-full shadow-sm shadow-rose-500/50"></div>
          </div>
        ) : (
          <span className="text-slate-300 dark:text-slate-600 font-bold tracking-widest italic text-[10px] uppercase opacity-50">
            {t("ess.attendance.active_status")}
          </span>
        )}
      </td>
      <td className="px-8 py-6 text-center">
        {entry.duration_minutes ? (
          <span className="inline-flex items-center px-3 py-1 rounded-xl bg-slate-100 dark:bg-slate-700/50 text-slate-600 dark:text-slate-300 text-[10px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700/50 shadow-inner">
            {Math.floor(entry.duration_minutes / 60)}h {entry.duration_minutes % 60}m
          </span>
        ) : (
          <span className="text-slate-300 dark:text-slate-600 font-bold">-</span>
        )}
      </td>
      <td className="px-8 py-6 text-center">
        <OtStatusCell
          setClaimOpen={setClaimOpen}
          setSelectedSummary={setSelectedSummary}
          summary={summary}
        />
      </td>
      <td className="px-10 py-6 text-right">
        <div className="flex items-center justify-end gap-4">
          <span
            className={cn(
              "px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
              entry.punch_out_time
                ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                : "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800 animate-pulse"
            )}
          >
            {entry.punch_out_time
              ? t("common.completed").toUpperCase()
              : t("common.pending").toUpperCase()}
          </span>
          <MetadataPopover entry={entry} />
        </div>
      </td>
    </tr>
  );
}
