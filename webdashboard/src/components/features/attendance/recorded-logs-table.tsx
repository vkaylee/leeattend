import { Clock, CalendarRange, Copy, User } from 'lucide-react';
import React, { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import InfiniteScrollSentinel from '@/components/common/infinite-scroll-sentinel';
import SmartTable, { type SmartTableColumnMeta } from '@/components/ui/smart-table';
import StatusBadge from '@/components/ui/status-badge';
import { useToast } from "@/components/ui/toast";
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
  const toast = useToast();

  const columns = useMemo(() => [
    {
      id: "time",
      header: t("common.time"),
      cell: ({ row }: any) => {
        const log = row.original;
        return (
          <div className="flex items-center gap-4">
            <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50">
              <Clock size={18} />
            </div>
            <div>
              <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">
                {formatDate(log.punch_time)}
              </div>
              <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">
                {formatTime(log.punch_time)}
              </div>
            </div>
          </div>
        );
      },
      size: 200,
      meta: {
        sticky: "left"
      } as SmartTableColumnMeta
    },
    {
      id: "employee",
      header: t("common.employee"),
      cell: ({ row }: any) => {
        const log = row.original;
        return (
          <div>
            <div className="font-bold text-slate-800 dark:text-slate-100 text-sm tracking-tight uppercase">
              {getEmployeeName(log)}
            </div>
            <div className="text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest mt-0.5">
              {getEmployeeID(log) || t("common.system")}
            </div>
          </div>
        );
      },
      size: 240,
      meta: {
        sticky: "left"
      } as SmartTableColumnMeta
    },
    {
      id: "source",
      header: t("common.source"),
      cell: ({ row }: any) => {
        const log = row.original;
        const deviceName = log.device_name ?? (log.source === "Device" && getDeviceName(log.device_id));
        return (
          <div className="flex items-center gap-2">
            <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700">
              {log.source}
            </span>
            {!!deviceName && (
              <span className="text-[10px] font-bold text-slate-700 dark:text-slate-200 uppercase truncate">
                {deviceName}
              </span>
            )}
          </div>
        );
      },
      size: 240
    },
    {
      id: "status",
      header: () => <div className="text-right w-full">{t("common.status")}</div>,
      cell: () => (
        <div className="flex justify-end w-full">
          <StatusBadge variant="success">{t("attendance_logs.status.recorded")}</StatusBadge>
        </div>
      ),
      size: 160,
      meta: {
        headerClassName: "text-right",
        cellClassName: "text-right"
      } as SmartTableColumnMeta
    }
  ], [t, formatDate, formatTime, getEmployeeName, getEmployeeID, getDeviceName]);

  // Context Menu items for Logs context action
  const contextMenuItems = (log: RawAttendanceLog) => [
    {
      label: t("attendance_logs.context.copy_id", "Copy Employee ID"),
      icon: <User size={14} />,
      onClick: () => {
        const empId = getEmployeeID(log);
        if (empId) {
          navigator.clipboard.writeText(empId);
          toast.success(t("common.copied", "Copied to clipboard"), empId);
        }
      },
      disabled: !getEmployeeID(log)
    },
    {
      label: t("attendance_logs.context.copy_time", "Copy Punch Time"),
      icon: <Clock size={14} />,
      onClick: () => {
        const pTime = formatTime(log.punch_time);
        navigator.clipboard.writeText(`${formatDate(log.punch_time)} ${pTime}`);
        toast.success(t("common.copied", "Copied to clipboard"), pTime);
      }
    }
  ];

  // Footer sentinel row for Infinite Scroll
  const footerRow = useMemo(() => (
    <InfiniteScrollSentinel 
      colSpan={4} 
      isFetchingNextPage={isFetchingNextPage} 
      sentinelRef={loadMoreRef} 
    />
  ), [isFetchingNextPage, loadMoreRef]);

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden h-[600px] flex flex-col p-6">
      <div className="flex-1 min-h-0">
        <SmartTable<RawAttendanceLog>
          data={logs}
          columns={columns}
          getRowId={(log) => log.id}
          isLoading={isLoading}
          contextMenuItems={contextMenuItems}
          footerRow={footerRow}
          emptyMessage={t("attendance_logs.messages.no_data")}
        />
      </div>
    </div>
  );
}
