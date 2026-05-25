import { Clock, CalendarRange } from 'lucide-react';
import React from 'react';
import { useTranslation } from 'react-i18next';

import InfiniteScrollSentinel from '@/components/common/infinite-scroll-sentinel';
import { TableSkeleton } from '@/components/common/table-skeleton';
import StatusBadge from '@/components/ui/status-badge';
import { Typography } from "@/components/ui/typography";
import { RawAttendanceLog } from '@/lib/api/types';

interface RecordedLogsTableProps {
  formatDate: (date: string | null | undefined) => string;
  formatTime: (date: string | null | undefined) => string;
  getDeviceName: (deviceId?: string | null) => string;
  getEmployeeID: (log: RawAttendanceLog) => string;
  getEmployeeName: (log: RawAttendanceLog) => string;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  loadMoreRef: React.RefObject<HTMLTableRowElement | null>;
  logs: RawAttendanceLog[];
}

export function RecordedLogsTable({
  logs,
  isLoading,
  isFetchingNextPage,
  loadMoreRef,
  getEmployeeName,
  getEmployeeID,
  getDeviceName,
  formatDate,
  formatTime
}: RecordedLogsTableProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <thead className="sticky top-0 z-10 bg-white/80 dark:bg-slate-900/80 backdrop-blur pb-2">
            <tr className="bg-slate-50/80 dark:bg-slate-800/80 border-b border-slate-100 dark:border-slate-800 shadow-sm backdrop-blur">
              <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.time")}</th>
              <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
              <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.source")}</th>
              <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.status")}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <TableSkeleton rows={5} variant="primary" />
            ) : logs.length === 0 ? (
              <tr>
                <td className="px-8 py-24 text-center" colSpan={4}>
                  <div className="flex flex-col items-center justify-center space-y-4">
                    <div className="w-16 h-16 bg-slate-50 dark:bg-slate-800/50 rounded-full flex items-center justify-center text-slate-300 dark:text-slate-600">
                      <CalendarRange size={32} />
                    </div>
                    <div>
                      <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300 uppercase tracking-widest" variant="h3">{t("attendance_logs.messages.no_data")}</Typography>
                      <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-2 max-w-sm mx-auto font-medium" variant="p">{t("attendance_logs.messages.no_data_hint")}</Typography>
                    </div>
                  </div>
                </td>
              </tr>
            ) : (
              <>
                {logs.map((log) => (
                  <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group" key={log.id}>
                    <td className="px-8 py-5">
                      <div className="flex items-center gap-4">
                        <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50"><Clock size={18} /></div>
                        <div>
                          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
                          <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="font-bold text-slate-800 dark:text-slate-100 text-sm tracking-tight uppercase">{getEmployeeName(log)}</div>
                      <div className="text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest mt-0.5">{getEmployeeID(log) || t("common.system")}</div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-2">
                          <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700">{log.source}</span>
                          {!!(log.device_name ?? (log.source === "Device" && getDeviceName(log.device_id))) && <span className="text-[10px] font-bold text-slate-700 dark:text-slate-200 uppercase">{log.device_name ?? getDeviceName(log.device_id)}</span>}
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5 text-right">
                      <StatusBadge variant="success">{t("attendance_logs.status.recorded")}</StatusBadge>
                    </td>
                  </tr>
                ))}
                <InfiniteScrollSentinel colSpan={4} isFetchingNextPage={isFetchingNextPage} sentinelRef={loadMoreRef} />
              </>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
