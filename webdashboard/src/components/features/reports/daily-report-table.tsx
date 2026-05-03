import React from "react";

import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";

import { DailyReportTableRow } from "./daily-report-table-row";

export interface DailyReportTableProps {
  adjustmentMap: Record<string, AttendanceAdjustmentRequest>;
  confirmingRow: string | null;
  data: DailyAttendanceReport[];
  expandedDebugRows: Set<string>;
  formatDate: (date: string) => string;
  formatTime: (time: string | null) => string;
  hideEmployeeName?: boolean;
  isDebugMode: boolean;
  isLoading: boolean;
  onApproveOt?: (row: DailyAttendanceReport, action: "approved" | "rejected") => void;
  onConfirmClick: (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => void;
  onReprocessClick: (row: DailyAttendanceReport) => void;
  onResolveUnscheduled?: (row: DailyAttendanceReport) => void;
  onToggleExpandRow: (rowKey: string) => void;
  reprocessingRow: string | null;
  t: (key: string) => string;
  workUnitMinutes: number;
}

function TableHeader({ hideEmployeeName, isDebugMode, t }: { hideEmployeeName: boolean; isDebugMode: boolean; t: (k: string) => string }) {
  return (
    <thead>
      <tr className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky top-0 z-20">
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest sticky left-0 z-20 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm">
          {t("common.date")}
        </th>
        {!hideEmployeeName ? (
          <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest sticky left-[80px] sm:left-[100px] z-20 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm">
            {t("common.employee")}
          </th>
        ) : null}
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.shift")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.in")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.out")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("reports.table.std_hours")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-orange-500 dark:text-orange-400 uppercase tracking-widest text-right">{t("reports.table.ot_hours")}</th>
        <th className="px-4 py-2.5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.status")}</th>
        {isDebugMode ? (
          <th className="px-4 py-2.5 text-[10px] font-bold text-amber-600 dark:text-amber-300 uppercase tracking-widest text-center">DEBUG</th>
        ) : null}
        <th className="px-4 py-2.5"></th>
      </tr>
    </thead>
  );
}

export default function DailyReportTable({
  adjustmentMap,
  confirmingRow,
  data,
  expandedDebugRows,
  formatDate,
  formatTime,
  hideEmployeeName = false,
  isDebugMode,
  isLoading,
  onApproveOt,
  onConfirmClick,
  onReprocessClick,
  onResolveUnscheduled,
  onToggleExpandRow,
  reprocessingRow,
  t,
  workUnitMinutes,
}: DailyReportTableProps) {
  const colSpan = isDebugMode ? (hideEmployeeName ? 9 : 10) : (hideEmployeeName ? 8 : 9);

  return (
    <div className="overflow-auto flex-1 min-h-0">
      <table className="w-full text-left min-w-[1200px] border-collapse relative">
        <TableHeader hideEmployeeName={hideEmployeeName} isDebugMode={isDebugMode} t={t} />
        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {isLoading ? (
            <tr>
              <td className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse" colSpan={colSpan}>
                {t("common.loading")}
              </td>
            </tr>
          ) : data.length === 0 ? (
            <tr>
              <td className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50" colSpan={colSpan}>
                {t("reports.messages.no_data")}
              </td>
            </tr>
          ) : (
            data.map((row: DailyAttendanceReport) => (
              <DailyReportTableRow
                adjustmentMap={adjustmentMap}
                confirmingRow={confirmingRow}
                expandedDebugRows={expandedDebugRows}
                formatDate={formatDate}
                formatTime={formatTime}
                hideEmployeeName={hideEmployeeName}
                isDebugMode={isDebugMode}
                key={`${row.employee_id}-${row.date}`}
                onApproveOt={onApproveOt}
                onConfirmClick={onConfirmClick}
                onReprocessClick={onReprocessClick}
                onResolveUnscheduled={onResolveUnscheduled}
                onToggleExpandRow={onToggleExpandRow}
                reprocessingRow={reprocessingRow}
                row={row}
                t={t}
                workUnitMinutes={workUnitMinutes}
              />
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
