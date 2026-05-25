import { History } from "lucide-react";
import { useTranslation } from "react-i18next";

import { TimeEntryRow } from "@/components/time-tracking/time-entry-row";
import { Typography } from "@/components/ui/typography";
import { AttendanceEntry } from "@/lib/api/types";

interface HistoryTableProps {
  entries: AttendanceEntry[];
}

export function HistoryTable({ entries }: HistoryTableProps) {
  const { t } = useTranslation();

  return (
    <div className="lg:col-span-7 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[40px] overflow-hidden shadow-sm flex flex-col">
      <div className="px-8 py-6 border-b border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <History className="w-5 h-5 text-primary-500" />
          <Typography className="font-bold text-slate-800 dark:text-slate-100 uppercase tracking-widest text-xs" variant="h3">
            {t("ess.clock.recent_history")}
          </Typography>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto max-h-[600px]">
        <table className="w-full text-left border-collapse">
          <thead className="sticky top-0 bg-white dark:bg-slate-900/80 backdrop-blur-md z-10 border-b border-slate-100 dark:border-slate-800">
            <tr>
              <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                {t("common.date")}
              </th>
              <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                {t("common.in")}
              </th>
              <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                {t("common.out")}
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {entries.length === 0 ? (
              <tr>
                <td
                  className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50"
                  colSpan={3}
                >
                  {t("ess.clock.no_entries")}
                </td>
              </tr>
            ) : (
              entries.map((entry) => (
                <TimeEntryRow
                  entry={entry}
                  key={entry.id}
                  showAdjustButton={false}
                />
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
