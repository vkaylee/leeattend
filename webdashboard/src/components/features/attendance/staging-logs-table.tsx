import { AlertTriangle, Check, Link2, UserPlus, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { Department } from '@/lib/api/types';

import { EmployeeSearch } from '@/components/common/employee-search';
import InfiniteScrollSentinel from '@/components/common/infinite-scroll-sentinel';
import { TableSkeleton } from '@/components/common/table-skeleton';
import Button from '@/components/ui/button';
import ResponsiveTable from '@/components/ui/responsive-table';
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";
import { StagingLog } from '@/lib/api/attendance';

interface StagingLogsTableProps {
  departments?: Department[];
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

interface StagingLogRowProps {
  assigningLogId: string | null;
  departments?: Department[];
  formatDate: (date: string | null | undefined) => string;
  formatTime: (date: string | null | undefined) => string;
  getDeviceName: (deviceId?: string | null) => string;
  isDismissing: boolean;
  log: StagingLog;
  onCreateEmployee: (rawUserId: string) => void;
  onDismiss: (logId: string) => void;
  onResolve: (logId: string, employeeId: string) => void;
  setAssigningLogId: (id: string | null) => void;
  wsId: string;
}

function StagingLogRow({
  log,
  assigningLogId,
  setAssigningLogId,
  wsId,
  departments,
  getDeviceName,
  formatDate,
  formatTime,
  onResolve,
  onDismiss,
  isDismissing,
  onCreateEmployee
}: StagingLogRowProps) {
  const { t } = useTranslation();

  return (
    <TableRow className="hover:bg-amber-50 dark:bg-amber-950/30 transition-colors group" key={log.id}>
      <TableCell className="px-8 py-5">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-xl bg-amber-50 dark:bg-amber-950 flex items-center justify-center text-amber-600 dark:text-amber-300 shadow-inner border border-amber-100 dark:border-amber-800/50"><AlertTriangle size={18} /></div>
          <div>
            <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
            <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
          </div>
        </div>
      </TableCell>
      <TableCell className="px-8 py-5">
        <div className="font-bold text-amber-700 dark:text-amber-300 text-lg tracking-tight font-mono">{log.raw_user_id}</div>
        <div className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5">
          {log.status === "ambiguous"
            ? t("attendance_logs.ambiguous_employment")
            : t("attendance_logs.not_in_system")}
        </div>
      </TableCell>
      <TableCell className="px-8 py-5">
        <div className="flex flex-col gap-1">
          {getDeviceName(log.device_id) ? (
            <span className="text-sm font-bold text-slate-700 dark:text-slate-200">{getDeviceName(log.device_id)}</span>
          ) : log.device_id ? (
            <span className="text-xs font-mono text-slate-500 dark:text-slate-400">{log.device_id}</span>
          ) : (
            <span className="text-xs text-slate-400 dark:text-slate-500 italic">-</span>
          )}
        </div>
      </TableCell>
      <TableCell className="px-8 py-5">
        <div className="flex items-center justify-end gap-2">
          {assigningLogId === log.id ? (
            <div className="flex items-center gap-2 min-w-[200px]">
              <EmployeeSearch
                departments={departments}
                onChange={(value) => {
                  const empId = Array.isArray(value) ? value[0] : value;
                  if (empId) {
                    onResolve(log.id, empId);
                    setAssigningLogId(null);
                  }
                }}
                placeholder={t("attendance_logs.select_employee")}
                value=""
                workspaceId={wsId}
              />
              <Button
                aria-label={t("common.cancel")}
                className="p-2 text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 transition-colors h-9 w-9"
                onClick={() => { setAssigningLogId(null); }}
                size="icon"
                variant="ghost"
              >
                ✕
              </Button>
            </div>
          ) : (
            <>
              <Button
                className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-primary-100 dark:bg-primary-900 transition-all h-auto"
                onClick={() => { setAssigningLogId(log.id); }}
                variant="ghost"
              >
                <Link2 size={12} /> {t("attendance_logs.assign")}
              </Button>
              <Button
                className="flex items-center gap-1.5 px-3 py-1.5 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-emerald-100 dark:bg-emerald-900 transition-all h-auto"
                onClick={() => { onCreateEmployee(log.raw_user_id); }}
                variant="ghost"
              >
                <UserPlus size={12} /> {t("attendance_logs.create_employee")}
              </Button>
              <Button
                className="flex items-center gap-1.5 px-3 py-1.5 bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-rose-100 dark:bg-rose-900 transition-all disabled:opacity-50 h-auto"
                disabled={isDismissing}
                onClick={() => { onDismiss(log.id); }}
                variant="ghost"
              >
                <Trash2 size={12} /> {t("common.delete")}
              </Button>
            </>
          )}
        </div>
      </TableCell>
    </TableRow>
  );
}

// ── Mobile Card ────────────────────────────────────────────────────

function StagingLogMobileCard({
  log,
  assigningLogId,
  setAssigningLogId,
  wsId,
  departments,
  getDeviceName,
  formatDate,
  formatTime,
  onResolve,
  onDismiss,
  isDismissing,
  onCreateEmployee,
}: StagingLogRowProps) {
  const { t } = useTranslation();

  return (
    <div className="rounded-2xl border border-amber-100 bg-amber-50/30 p-5 shadow-sm transition-all active:scale-[0.99] dark:border-amber-800/40 dark:bg-amber-950/20">
      {/* Header: Time + Warning icon */}
      <div className="flex items-start gap-3">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-amber-100 bg-amber-50 text-amber-600 shadow-inner dark:border-amber-800/50 dark:bg-amber-950 dark:text-amber-300">
          <AlertTriangle size={18} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="text-sm font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200">{formatDate(log.punch_time)}</div>
          <div className="mt-1 font-mono text-xs font-bold text-slate-500 dark:text-slate-400">{formatTime(log.punch_time)}</div>
        </div>
      </div>

      {/* Unknown ID */}
      <div className="mt-4 border-t border-amber-100/50 pt-4 dark:border-amber-800/30">
        <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t("attendance_logs.unknown_id")}</div>
        <div className="mt-1 font-mono text-lg font-bold tracking-tight text-amber-700 dark:text-amber-300">{log.raw_user_id}</div>
        <div className="mt-0.5 text-[9px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("attendance_logs.not_in_system")}</div>
      </div>

      {/* Device */}
      <div className="mt-3">
        <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t("common.device")}</div>
        <div className="mt-1">
          {getDeviceName(log.device_id) ? (
            <span className="text-sm font-bold text-slate-700 dark:text-slate-200">{getDeviceName(log.device_id)}</span>
          ) : log.device_id ? (
            <span className="font-mono text-xs text-slate-500 dark:text-slate-400">{log.device_id}</span>
          ) : (
            <span className="text-xs italic text-slate-400 dark:text-slate-500">-</span>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className="mt-4 border-t border-amber-100/50 pt-4 dark:border-amber-800/30">
        {assigningLogId === log.id ? (
          <div className="flex items-center gap-2">
            <EmployeeSearch
              departments={departments}
              onChange={(value) => {
                const empId = Array.isArray(value) ? value[0] : value;
                if (empId) {
                  onResolve(log.id, empId);
                  setAssigningLogId(null);
                }
              }}
              placeholder={t("attendance_logs.select_employee")}
              value=""
              workspaceId={wsId}
            />
            <Button
              aria-label={t("common.cancel")}
              className="p-2 text-slate-400 transition-colors hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300"
              onClick={() => { setAssigningLogId(null); }}
              size="icon"
              variant="ghost"
            >
              ✕
            </Button>
          </div>
        ) : (
          <div className="flex flex-wrap gap-2">
            <Button
              className="flex items-center gap-1.5 rounded-lg bg-primary-50 px-3 py-2 text-[9px] font-bold uppercase tracking-widest text-primary-600 transition-all hover:bg-primary-100 dark:bg-primary-950 dark:text-primary-400 dark:hover:bg-primary-900"
              onClick={() => { setAssigningLogId(log.id); }}
              variant="ghost"
            >
              <Link2 size={12} /> {t("attendance_logs.assign")}
            </Button>
            <Button
              className="flex items-center gap-1.5 rounded-lg bg-emerald-50 px-3 py-2 text-[9px] font-bold uppercase tracking-widest text-emerald-600 transition-all hover:bg-emerald-100 dark:bg-emerald-950 dark:text-emerald-300 dark:hover:bg-emerald-900"
              onClick={() => { onCreateEmployee(log.raw_user_id); }}
              variant="ghost"
            >
              <UserPlus size={12} /> {t("attendance_logs.create_employee")}
            </Button>
            <Button
              className="flex items-center gap-1.5 rounded-lg bg-rose-50 px-3 py-2 text-[9px] font-bold uppercase tracking-widest text-rose-600 transition-all hover:bg-rose-100 disabled:opacity-50 dark:bg-rose-950 dark:text-rose-300 dark:hover:bg-rose-900"
              disabled={isDismissing}
              onClick={() => { onDismiss(log.id); }}
              variant="ghost"
            >
              <Trash2 size={12} /> {t("common.delete")}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}

export function StagingLogsTable({
  logs,
  isLoading,
  isFetchingNextPage,
  loadMoreRef,
  wsId,
  departments,
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

  return (
    <ResponsiveTable<StagingLog>
      data={logs}
      emptyState={{
        icon: <Check className="h-10 w-10" />,
        title: t("attendance_logs.messages.no_unknown_logs"),
      }}
      getCardKey={(log) => log.id}
      isLoading={isLoading}
      renderCard={(log) => (
        <StagingLogMobileCard
          assigningLogId={assigningLogId}
          departments={departments}
          formatDate={formatDate}
          formatTime={formatTime}
          getDeviceName={getDeviceName}
          isDismissing={isDismissing}
          log={log}
          onCreateEmployee={onCreateEmployee}
          onDismiss={onDismiss}
          onResolve={onResolve}
          setAssigningLogId={setAssigningLogId}
          wsId={wsId}
        />
      )}
      renderDesktopTable={() => (
        <Table>
          <TableHeader className="sticky top-0 z-10 bg-white/80 dark:bg-slate-900/80 backdrop-blur pb-2">
            <TableRow className="bg-amber-50/80 dark:bg-amber-950/80 border-b border-amber-100 dark:border-amber-800 shadow-sm backdrop-blur">
              <TableHead className="px-8 py-4 text-amber-700 dark:text-amber-300">{t("common.time")}</TableHead>
              <TableHead className="px-8 py-4 text-amber-700 dark:text-amber-300">{t("attendance_logs.unknown_id")}</TableHead>
              <TableHead className="px-8 py-4 text-amber-700 dark:text-amber-300">{t("common.device")}</TableHead>
              <TableHead className="px-8 py-4 text-amber-700 dark:text-amber-300 text-right">{t("common.actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableSkeleton rows={3} variant="warning" />
            ) : logs.length === 0 ? (
              <TableRow>
                <TableCell className="px-8 py-24 text-center" colSpan={4}>
                  <div className="flex flex-col items-center justify-center space-y-4">
                    <div className="w-16 h-16 bg-emerald-50 dark:bg-emerald-950/30 rounded-full flex items-center justify-center text-emerald-500/50">
                      <Check className="h-8 w-8 text-emerald-400" />
                    </div>
                    <div>
                      <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300 uppercase tracking-widest" variant="h3">{t("attendance_logs.messages.no_unknown_logs")}</Typography>
                      <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-2 font-medium" variant="p">{t("attendance_logs.messages.all_matched")}</Typography>
                    </div>
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              <>
                {logs.map((log) => (
                  <StagingLogRow
                    assigningLogId={assigningLogId}
                    departments={departments}
                    formatDate={formatDate}
                    formatTime={formatTime}
                    getDeviceName={getDeviceName}
                    isDismissing={isDismissing}
                    key={log.id}
                    log={log}
                    onCreateEmployee={onCreateEmployee}
                    onDismiss={onDismiss}
                    onResolve={onResolve}
                    setAssigningLogId={setAssigningLogId}
                    wsId={wsId}
                  />
                ))}
                <InfiniteScrollSentinel colSpan={4} isFetchingNextPage={isFetchingNextPage} sentinelRef={loadMoreRef} />
              </>
            )}
          </TableBody>
        </Table>
      )}
    />
  );
}

