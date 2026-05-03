import React from "react";
import { useTranslation } from "react-i18next";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { AttendanceDailySummary, AttendanceEntry } from "@/lib/api/types";

import { AttendanceRow } from "./attendance-table/row";

interface AttendanceTableProps {
  entries: AttendanceEntry[];
  getSummary: (date: string) => AttendanceDailySummary | undefined;
  setClaimOpen: (open: boolean) => void;
  setSelectedSummary: (summary: AttendanceDailySummary) => void;
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
    <div className="overflow-x-auto">
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
            <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
              {t("ess.attendance.date")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("common.in")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("common.out")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("ess.attendance.duration")}
            </th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
              {t("ess.attendance.ot_status")}
            </th>
            <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right">
              {t("common.status")}
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {entries.map((entry) => (
            <AttendanceRow
              entry={entry}
              formatDate={formatDate}
              formatTime={formatTime}
              getSummary={getSummary}
              key={entry.id}
              setClaimOpen={setClaimOpen}
              setSelectedSummary={setSelectedSummary}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}
