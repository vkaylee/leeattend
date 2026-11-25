import { AlertTriangle, Check, Link2, Trash2, UserPlus } from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { Department } from '@/lib/api/types';

import { EmployeeSearch } from '@/components/common/employee-search';
import InfiniteScrollSentinel from '@/components/common/infinite-scroll-sentinel';
import { TableSkeleton } from '@/components/common/table-skeleton';
import Button from '@/components/ui/button';
import ResponsiveTable from '@/components/ui/responsive-table';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Typography } from '@/components/ui/typography';
import { StagingLog } from '@/lib/api/attendance';

interface StagingLogsTableProps {
  departments?: Department[];
  formatDate: (date: string | null | undefined) => string;
  formatTime: (date: string | null | undefined) => string;
  isDismissing: boolean;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  loadMoreRef: React.RefObject<HTMLTableRowElement | null>;
  logs: StagingLog[];
  onCreateEmployee: (rawUserId: string, logId?: string) => void;
  onDismiss: (logId: string) => void;
  onResolve: (logId: string, employeeId: string) => void;
}

type AssignmentProps = Pick<StagingLogsTableProps, 'departments' | 'isDismissing' | 'onCreateEmployee' | 'onDismiss' | 'onResolve'> & {
  assigningLogId: string | null;
  log: StagingLog;
  setAssigningLogId: (id: string | null) => void;
};

function AssignmentActions({ assigningLogId, departments, isDismissing, log, onCreateEmployee, onDismiss, onResolve, setAssigningLogId}: AssignmentProps) {
  const { t } = useTranslation();

  if (assigningLogId === log.id) {
    return (
      <div className="flex items-center gap-2">
        <EmployeeSearch departments={departments}
        onChange={(value) => {
          const employeeId = Array.isArray(value) ? value[0] : value;
          if (employeeId) {
            onResolve(log.id, employeeId);
            setAssigningLogId(null);
          }
        }}
        placeholder={t('attendance_logs.select_employee')}
        value=""  />
        <Button aria-label={t('common.cancel')} onClick={() => { setAssigningLogId(null); }} size="icon" variant="ghost">✕</Button>
      </div>
    );
  }

  return (
    <div className="flex flex-wrap gap-2">
      <Button onClick={() => { setAssigningLogId(log.id); }} variant="ghost"><Link2 size={12} /> {t('attendance_logs.assign')}</Button>
      <Button onClick={() => { onCreateEmployee(log.raw_user_id, log.id); }} variant="ghost"><UserPlus size={12} /> {t('attendance_logs.create_employee')}</Button>
      <Button disabled={isDismissing} onClick={() => { onDismiss(log.id); }} variant="ghost"><Trash2 size={12} /> {t('common.delete')}</Button>
    </div>
  );
}

function StagingLogRow({ assigningLogId, departments, formatDate, formatTime, isDismissing, log, onCreateEmployee, onDismiss, onResolve, setAssigningLogId}: AssignmentProps & Pick<StagingLogsTableProps, 'formatDate' | 'formatTime'>) {
  const { t } = useTranslation();

  return (
    <TableRow className="group transition-colors hover:bg-amber-50 dark:bg-amber-950/30" key={log.id}>
      <TableCell className="px-8 py-5">
        <div className="flex items-center gap-4">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-amber-100 bg-amber-50 text-amber-600 dark:border-amber-800/50 dark:bg-amber-950 dark:text-amber-300"><AlertTriangle size={18} /></div>
          <div><div className="text-sm font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200">{formatDate(log.punch_time)}</div><div className="font-mono text-[10px] font-bold uppercase tracking-tighter text-slate-500 dark:text-slate-400">{formatTime(log.punch_time)}</div></div>
        </div>
      </TableCell>
      <TableCell className="px-8 py-5"><div className="font-mono text-lg font-bold tracking-tight text-amber-700 dark:text-amber-300">{log.raw_user_id}</div><div className="mt-0.5 text-[9px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{log.status === 'ambiguous' ? t('attendance_logs.ambiguous_employment') : t('attendance_logs.not_in_system')}</div></TableCell>
      <TableCell className="px-8 py-5"><span className="text-xs text-slate-500 dark:text-slate-400">{t('attendance_logs.messages.unknown_source', 'External attendance source')}</span></TableCell>
      <TableCell className="px-8 py-5"><div className="flex justify-end"><AssignmentActions assigningLogId={assigningLogId} departments={departments} isDismissing={isDismissing} log={log} onCreateEmployee={onCreateEmployee} onDismiss={onDismiss} onResolve={onResolve} setAssigningLogId={setAssigningLogId}  /></div></TableCell>
    </TableRow>
  );
}

function StagingLogMobileCard({ assigningLogId, departments, formatDate, formatTime, isDismissing, log, onCreateEmployee, onDismiss, onResolve, setAssigningLogId}: AssignmentProps & Pick<StagingLogsTableProps, 'formatDate' | 'formatTime'>) {
  const { t } = useTranslation();

  return (
    <div className="rounded-2xl border border-amber-100 bg-amber-50/30 p-5 shadow-sm dark:border-amber-800/40 dark:bg-amber-950/20">
      <div className="flex items-start gap-3"><div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-amber-100 bg-amber-50 text-amber-600 dark:border-amber-800/50 dark:bg-amber-950 dark:text-amber-300"><AlertTriangle size={18} /></div><div className="min-w-0 flex-1"><div className="text-sm font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200">{formatDate(log.punch_time)}</div><div className="mt-1 font-mono text-xs font-bold text-slate-500 dark:text-slate-400">{formatTime(log.punch_time)}</div></div></div>
      <div className="mt-4 border-t border-amber-100/50 pt-4 dark:border-amber-800/30"><div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t('attendance_logs.unknown_id')}</div><div className="mt-1 font-mono text-lg font-bold tracking-tight text-amber-700 dark:text-amber-300">{log.raw_user_id}</div><div className="mt-0.5 text-[9px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t('attendance_logs.not_in_system')}</div></div>
      <div className="mt-3"><div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t('common.source')}</div><div className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t('attendance_logs.messages.unknown_source', 'External attendance source')}</div></div>
      <div className="mt-4 border-t border-amber-100/50 pt-4 dark:border-amber-800/30"><AssignmentActions assigningLogId={assigningLogId} departments={departments} isDismissing={isDismissing} log={log} onCreateEmployee={onCreateEmployee} onDismiss={onDismiss} onResolve={onResolve} setAssigningLogId={setAssigningLogId}  /></div>
    </div>
  );
}

export function StagingLogsTable({ departments, formatDate, formatTime, isDismissing, isFetchingNextPage, isLoading, loadMoreRef, logs, onCreateEmployee, onDismiss, onResolve}: StagingLogsTableProps) {
  const { t } = useTranslation();
  const [assigningLogId, setAssigningLogId] = useState<string | null>(null);
  const assignmentProps = (log: StagingLog) => ({ assigningLogId, departments, formatDate, formatTime, isDismissing, log, onCreateEmployee, onDismiss, onResolve, setAssigningLogId});

  return (
    <ResponsiveTable<StagingLog>
      data={logs}
      emptyState={{ icon: <Check className="h-10 w-10" />, title: t('attendance_logs.messages.no_unknown_logs') }}
      getCardKey={(log) => log.id}
      isLoading={isLoading}
      renderCard={(log) => <StagingLogMobileCard {...assignmentProps(log)} />}
      renderDesktopTable={() => (
        <Table className="min-w-[900px]">
          <TableHeader><TableRow><TableHead>{t('common.time')}</TableHead><TableHead>{t('attendance_logs.unknown_id')}</TableHead><TableHead>{t('common.source')}</TableHead><TableHead className="text-right">{t('common.actions')}</TableHead></TableRow></TableHeader>
          <TableBody>
            {isLoading ? <TableSkeleton rows={3} variant="warning" /> : logs.length === 0 ? <TableRow><TableCell className="px-8 py-24 text-center" colSpan={4}><Typography variant="p">{t('attendance_logs.messages.no_unknown_logs')}</Typography></TableCell></TableRow> : <>{logs.map((log) => <StagingLogRow key={log.id} {...assignmentProps(log)} />)}<InfiniteScrollSentinel colSpan={4} isFetchingNextPage={isFetchingNextPage} sentinelRef={loadMoreRef} /></>}
          </TableBody>
        </Table>
      )}
    />
  );
}
