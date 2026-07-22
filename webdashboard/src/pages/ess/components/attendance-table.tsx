import { Inbox } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import ResponsiveTable from "@/components/ui/responsive-table";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { AttendanceDailySummary, AttendanceEntry } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { MetadataPopover } from "./attendance-table/metadata-popover";
import { OtStatusCell } from "./attendance-table/ot-status-cell";

interface AttendanceTableProps {
  entries: AttendanceEntry[];
  getSummary: (date: string) => AttendanceDailySummary | undefined;
  setClaimOpen: (open: boolean) => void;
  setSelectedSummary: (summary: AttendanceDailySummary) => void;
}

function AttendanceMobileCard({
  entry,
  formatDate,
  formatTime,
  getSummary,
  setClaimOpen,
  setSelectedSummary,
}: {
  entry: AttendanceEntry;
  formatDate: (d: string) => string;
  formatTime: (d: string) => string;
  getSummary: (date: string) => AttendanceDailySummary | undefined;
  setClaimOpen: (open: boolean) => void;
  setSelectedSummary: (summary: AttendanceDailySummary) => void;
}) {
  const { t } = useTranslation();
  const dateString = format(new Date(entry.punch_in_time), "yyyy-MM-dd");
  const summary = getSummary(dateString);

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Date + Status */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 font-bold text-primary-600 shadow-inner dark:border-primary-800/50 dark:bg-primary-950 dark:text-primary-400">
            {format(new Date(entry.punch_in_time), "dd")}
          </div>
          <div className="min-w-0">
            <div className="text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
              {format(new Date(entry.punch_in_time), "EEEE")}
            </div>
            <div className="mt-0.5 text-[10px] font-bold uppercase tracking-[0.2em] text-slate-500 opacity-70 dark:text-slate-400">
              {formatDate(entry.punch_in_time)}
            </div>
          </div>
        </div>
        <span
          className={cn(
            "shrink-0 rounded-full border px-3 py-1 text-[9px] font-bold uppercase tracking-widest shadow-sm",
            entry.punch_out_time
              ? "border-emerald-100 bg-emerald-50 text-emerald-600 dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
              : "animate-pulse border-amber-100 bg-amber-50 text-amber-600 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300",
          )}
        >
          {entry.punch_out_time
            ? t("common.completed").toUpperCase()
            : t("common.pending").toUpperCase()}
        </span>
      </div>

      {/* Punch In/Out + Duration */}
      <div className="mt-4 grid grid-cols-3 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("common.in")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">{formatTime(entry.punch_in_time)}</div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("common.out")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">
            {entry.punch_out_time ? formatTime(entry.punch_out_time) : t("ess.attendance.active_status")}
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("ess.attendance.duration")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">
            {entry.duration_minutes ? `${Math.floor(entry.duration_minutes / 60)}h ${entry.duration_minutes % 60}m` : "—"}
          </div>
        </div>
      </div>

      {/* OT Status */}
      <div className="mt-3 flex items-center justify-between border-t border-slate-100 pt-4 dark:border-slate-800">
        <OtStatusCell
          setClaimOpen={setClaimOpen}
          setSelectedSummary={setSelectedSummary}
          summary={summary}
        />
        <MetadataPopover entry={entry} />
      </div>
    </div>
  );
}


export function AttendanceTable({
  entries,
  getSummary,
  setClaimOpen,
  setSelectedSummary,
}: AttendanceTableProps) {
  const { t } = useTranslation();
  const { formatDate, formatTime } = useDateTimeFormatter();

  return (
    <ResponsiveTable<AttendanceEntry>
      data={entries}
      emptyState={{
        icon: <Inbox className="h-8 w-8" />,
        title: t("ess.attendance.no_data"),
      }}
      getCardKey={(entry) => entry.id}
      renderCard={(entry) => (
        <AttendanceMobileCard
          entry={entry}
          formatDate={formatDate}
          formatTime={formatTime}
          getSummary={getSummary}
          setClaimOpen={setClaimOpen}
          setSelectedSummary={setSelectedSummary}
        />
      )}
      renderDesktopTable={() => (
        <div className="overflow-x-auto">
          <table className="w-full border-collapse text-left">
            <thead>
              <tr className="border-b border-slate-100 bg-slate-50 dark:border-slate-800 dark:bg-slate-800/50">
                <th className="px-10 py-6 text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400">
                  {t("ess.attendance.date")}
                </th>
                <th className="px-8 py-6 text-center text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400">
                  {t("common.in")}
                </th>
                <th className="px-8 py-6 text-center text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400">
                  {t("common.out")}
                </th>
                <th className="px-8 py-6 text-center text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400">
                  {t("ess.attendance.duration")}
                </th>
                <th className="px-8 py-6 text-center text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400">
                  {t("ess.attendance.ot_status")}
                </th>
                <th className="px-10 py-6 text-right text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400">
                  {t("common.status")}
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
              {entries.map((entry) => {
                const dateString = format(new Date(entry.punch_in_time), "yyyy-MM-dd");
                const summary = getSummary(dateString);

                return (
                  <tr
                    className="transition-all group hover:bg-slate-50 dark:hover:bg-slate-800/50"
                    key={entry.id}
                  >
                    <td className="px-10 py-6">
                      <div className="flex items-center gap-6">
                        <div className="flex h-14 w-14 items-center justify-center rounded-2xl border border-primary-100 bg-primary-50 font-bold text-2xl text-primary-600 shadow-inner transition-transform group-hover:scale-110 dark:border-primary-800/50 dark:bg-primary-950 dark:text-primary-400">
                          {format(new Date(entry.punch_in_time), "dd")}
                        </div>
                        <div>
                          <div className="mb-1.5 text-sm font-bold uppercase leading-none tracking-tight text-slate-700 dark:text-slate-200">
                            {format(new Date(entry.punch_in_time), "EEEE")}
                          </div>
                          <div className="text-[10px] font-bold uppercase leading-none tracking-[0.2em] text-slate-500 opacity-70 dark:text-slate-400">
                            {formatDate(entry.punch_in_time)}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-6 text-center">
                      <div className="flex flex-col items-center gap-2">
                        <span className="font-mono text-sm font-bold tracking-tighter text-slate-700 dark:text-slate-200">
                          {formatTime(entry.punch_in_time)}
                        </span>
                        <div className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-50 shadow-sm shadow-emerald-500/50 dark:bg-emerald-950" />
                      </div>
                    </td>
                    <td className="px-8 py-6 text-center">
                      {entry.punch_out_time ? (
                        <div className="flex flex-col items-center gap-2">
                          <span className="font-mono text-sm font-bold tracking-tighter text-slate-700 dark:text-slate-200">
                            {formatTime(entry.punch_out_time)}
                          </span>
                          <div className="h-1.5 w-1.5 rounded-full bg-rose-50 shadow-sm shadow-rose-500/50 dark:bg-rose-950" />
                        </div>
                      ) : (
                        <span className="text-[10px] font-bold italic uppercase tracking-widest text-slate-300 opacity-50 dark:text-slate-600">
                          {t("ess.attendance.active_status")}
                        </span>
                      )}
                    </td>
                    <td className="px-8 py-6 text-center">
                      {entry.duration_minutes ? (
                        <span className="inline-flex items-center rounded-xl border border-slate-200 bg-slate-100 px-3 py-1 text-[10px] font-bold uppercase tracking-widest text-slate-600 shadow-inner dark:border-slate-700/50 dark:bg-slate-700/50 dark:text-slate-300">
                          {Math.floor(entry.duration_minutes / 60)}h {entry.duration_minutes % 60}m
                        </span>
                      ) : (
                        <span className="font-bold text-slate-300 dark:text-slate-600">-</span>
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
                            "rounded-full border px-4 py-1.5 text-[9px] font-bold uppercase tracking-widest shadow-sm",
                            entry.punch_out_time
                              ? "border-emerald-100 bg-emerald-50 text-emerald-600 dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
                              : "animate-pulse border-amber-100 bg-amber-50 text-amber-600 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300",
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
              })}
            </tbody>
          </table>
        </div>
      )}
    />
  );
}
