import { Clock, History } from "lucide-react";
import { useTranslation } from "react-i18next";

import ResponsiveTable from "@/components/ui/responsive-table";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { AttendanceEntry } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface HistoryTableProps {
  entries: AttendanceEntry[];
}

function HistoryMobileCard({ entry }: { entry: AttendanceEntry }) {
  const { t } = useTranslation();
  const { formatDate, formatTime } = useDateTimeFormatter();

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div className="flex items-start gap-3">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400">
          <Clock size={18} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
            {formatDate(entry.punch_in_time)}
          </div>
          <div className="mt-1 space-y-1">
            <div className="flex items-center gap-2">
              <span className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("common.in")}</span>
              <span className="font-mono text-xs font-bold text-slate-700 dark:text-slate-200">{formatTime(entry.punch_in_time)}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("common.out")}</span>
              {entry.punch_out_time ? (
                <span className="font-mono text-xs font-bold text-slate-700 dark:text-slate-200">{formatTime(entry.punch_out_time)}</span>
              ) : (
                <span className="animate-pulse rounded-lg bg-amber-50 px-2 py-0.5 text-[9px] font-bold uppercase tracking-tighter text-amber-600 dark:bg-amber-950 dark:text-amber-300">
                  {t("time_tracking.status.working")}
                </span>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function HistoryTable({ entries }: HistoryTableProps) {
  const { t } = useTranslation();
  const { formatDate, formatTime } = useDateTimeFormatter();

  return (
    <div className="flex flex-col overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-900 sm:rounded-[40px]">
      <div className="flex items-center justify-between border-b border-slate-50 bg-slate-50 px-8 py-6 dark:border-slate-800 dark:bg-slate-800/30">
        <div className="flex items-center gap-3">
          <History className="h-5 w-5 text-primary-500" />
          <h3 className="text-xs font-bold uppercase tracking-widest text-slate-800 dark:text-slate-100">
            {t("ess.clock.recent_history")}
          </h3>
        </div>
      </div>

      <ResponsiveTable<AttendanceEntry>
        data={entries}
        emptyState={{
          icon: <Clock className="h-10 w-10" />,
          title: t("ess.clock.no_entries"),
        }}
        getCardKey={(entry) => entry.id}
        renderCard={(entry) => (
          <HistoryMobileCard entry={entry} />
        )}
        renderDesktopTable={() => (
          <div className="max-h-[600px] overflow-y-auto">
            <table className="w-full border-collapse text-left">
              <thead className="sticky top-0 z-10 border-b border-slate-100 bg-white/80 backdrop-blur-md dark:border-slate-800 dark:bg-slate-900/80">
                <tr>
                  <th className="px-8 py-4 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("common.date")}
                  </th>
                  <th className="px-8 py-4 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("common.in")}
                  </th>
                  <th className="px-8 py-4 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("common.out")}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                {entries.map((entry) => (
                  <tr className="transition-all group hover:bg-slate-50 dark:hover:bg-slate-800/50" key={entry.id}>
                    <td className="px-8 py-5">
                      <div className="flex items-center gap-3">
                        <span className="text-sm font-bold text-slate-700 dark:text-slate-200">
                          {formatDate(entry.punch_in_time)}
                        </span>
                        <span
                          className={cn(
                            "rounded-lg border px-2 py-0.5 text-[8px] font-bold uppercase tracking-tighter",
                            entry.entry_type === "break"
                              ? "border-amber-100 bg-amber-50 text-amber-600 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300"
                              : "border-primary-100 bg-primary-50 text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400",
                          )}
                        >
                          {entry.entry_type === "break" ? t("common.break") : t("common.work")}
                        </span>
                      </div>
                    </td>
                    <td className="px-8 py-5 font-mono text-xs font-bold text-slate-500 dark:text-slate-400">
                      {formatTime(entry.punch_in_time)}
                    </td>
                    <td className="px-8 py-5">
                      {entry.punch_out_time ? (
                        <span className="font-mono text-xs font-bold text-slate-500 dark:text-slate-400">
                          {formatTime(entry.punch_out_time)}
                        </span>
                      ) : (
                        <span
                          className={cn(
                            "animate-pulse rounded-lg px-2 py-1 text-[9px] font-bold uppercase tracking-tighter",
                            entry.entry_type === "break"
                              ? "bg-amber-50 text-amber-600 dark:bg-amber-950 dark:text-amber-300"
                              : "bg-primary-50 text-primary-600 dark:bg-primary-950 dark:text-primary-400",
                          )}
                        >
                          {entry.entry_type === "break"
                            ? t("time_tracking.status.on_break")
                            : t("time_tracking.status.working")}
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      />
    </div>
  );
}
