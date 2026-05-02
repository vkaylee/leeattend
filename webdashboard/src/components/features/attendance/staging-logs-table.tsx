import { AlertTriangle, Check, Link2, UserPlus, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { EmployeeSearch } from '@/components/common/employee-search';
import InfiniteScrollSentinel from '@/components/common/infinite-scroll-sentinel';
import { TableSkeleton } from '@/components/common/table-skeleton';
import { StagingLog } from '@/lib/api/attendance';

interface StagingLogsTableProps {
  formatDate: (date: string | null | undefined) => string;
  formatTime: (date: string | null | undefined) => string;
  getDeviceName: (deviceId?: string | null) => string;
  isDismissing: boolean;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  loadMoreRef: React.RefObject<HTMLTableRowElement | null>;
  logs: StagingLog[];
  onCreateEmployee: (rawUserId: string) => void;
  onDismiss: (logId: string) => void;
  onResolve: (logId: string, employeeId: string) => void;
  wsId: string;
}

export function StagingLogsTable({
  logs,
  isLoading,
  isFetchingNextPage,
  loadMoreRef,
  wsId,
  getDeviceName,
  formatDate,
  formatTime,
  onResolve,
  onDismiss,
  isDismissing,
  onCreateEmployee
}: StagingLogsTableProps) {
  const { t } = useTranslation();
  const [assigningLogId, setAssigningLogId] = useState<string | null>(null);

  const handleResolve = (logId: string, employeeId: string) => {
    onResolve(logId, employeeId);
    setAssigningLogId(null);
  };

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <thead className="sticky top-0 z-10 bg-white/80 dark:bg-slate-900/80 backdrop-blur pb-2">
            <tr className="bg-amber-50/80 dark:bg-amber-950/80 border-b border-amber-100 dark:border-amber-800 shadow-sm backdrop-blur">
              <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("common.time")}</th>
              <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("attendance_logs.unknown_id")}</th>
              <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("common.device")}</th>
              <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest text-right">{t("common.actions")}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <TableSkeleton rows={3} variant="warning" />
            ) : logs.length === 0 ? (
              <tr>
                <td className="px-8 py-24 text-center" colSpan={4}>
                  <div className="flex flex-col items-center justify-center space-y-4">
                    <div className="w-16 h-16 bg-emerald-50 dark:bg-emerald-950/30 rounded-full flex items-center justify-center text-emerald-500/50">
                      <Check className="h-8 w-8 text-emerald-400" />
                    </div>
                    <div>
                      <h3 className="text-sm font-bold text-slate-700 dark:text-slate-300 uppercase tracking-widest">{t("attendance_logs.messages.no_unknown_logs")}</h3>
                      <p className="text-xs text-slate-500 dark:text-slate-400 mt-2 font-medium">{t("attendance_logs.messages.all_matched")}</p>
                    </div>
                  </div>
                </td>
              </tr>
            ) : (
              <>
                {logs.map((log) => (
                  <tr className="hover:bg-amber-50 dark:bg-amber-950/30 transition-colors group" key={log.id}>
                    <td className="px-8 py-5">
                      <div className="flex items-center gap-4">
                        <div className="w-10 h-10 rounded-xl bg-amber-50 dark:bg-amber-950 flex items-center justify-center text-amber-600 dark:text-amber-300 shadow-inner border border-amber-100 dark:border-amber-800/50"><AlertTriangle size={18} /></div>
                        <div>
                          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
                          <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="font-bold text-amber-700 dark:text-amber-300 text-lg tracking-tight font-mono">{log.raw_user_id}</div>
                      <div className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5">{t("attendance_logs.not_in_system")}</div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="flex flex-col gap-1">
                        {getDeviceName(log.device_id) ? (
                          <span className="text-sm font-bold text-slate-700 dark:text-slate-200">{getDeviceName(log.device_id)}</span>
                        ) : log.device_id ? (
                          <span className="text-xs font-mono text-slate-500 dark:text-slate-400">{log.device_id}</span>
                        ) : (
                          <span className="text-xs text-slate-400 dark:text-slate-500 italic">-</span>
                        )}
                      </div>
                    </td>
                    <td className="px-8 py-5">
                      <div className="flex items-center justify-end gap-2">
                        {assigningLogId === log.id ? (
                          <div className="flex items-center gap-2 min-w-[200px]">
                            <EmployeeSearch
                              onChange={(value) => {
                                const empId = Array.isArray(value) ? value[0] : value;
                                if (empId) {
                                  handleResolve(log.id, empId);
                                }
                              }}
                              placeholder={t("attendance_logs.select_employee")}
                              value=""
                              workspaceId={wsId}
                            />
                            <button
                              className="p-2 text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600 transition-colors"
                              onClick={() => { setAssigningLogId(null); }}
                            >
                              ✕
                            </button>
                          </div>
                        ) : (
                          <>
                            <button
                              className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-primary-100 dark:bg-primary-900 transition-all"
                              onClick={() => { setAssigningLogId(log.id); }}
                            >
                              <Link2 size={12} /> {t("attendance_logs.assign")}
                            </button>
                            <button
                              className="flex items-center gap-1.5 px-3 py-1.5 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-emerald-100 dark:bg-emerald-900 transition-all"
                              onClick={() => { onCreateEmployee(log.raw_user_id); }}
                            >
                              <UserPlus size={12} /> {t("attendance_logs.create_employee")}
                            </button>
                            <button
                              className="flex items-center gap-1.5 px-3 py-1.5 bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-rose-100 dark:bg-rose-900 transition-all disabled:opacity-50"
                              disabled={isDismissing}
                              onClick={() => { onDismiss(log.id); }}
                            >
                              <Trash2 size={12} /> {t("common.delete")}
                            </button>
                          </>
                        )}
                      </div>
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
