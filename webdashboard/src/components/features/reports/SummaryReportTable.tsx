import { ArrowUp, ArrowDown, ArrowUpDown, ChevronRight, SearchX, Loader2 } from "lucide-react";
import { useEffect, RefObject } from "react";
import { AttendanceSummary, AttendanceReportSort } from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";
import { cn } from "@/lib/utils";

interface SummaryReportTableProps {
  data: AttendanceSummary[];
  isLoading: boolean;
  workUnitMinutes: number;
  sortBy: AttendanceReportSort | undefined;
  focusedRowIndex?: number;
  t: (key: string) => string;
  onSort: (field: 'name' | 'work_hours' | 'leave_days') => void;
  onEmployeeClick: (employeeId: string, employeeName: string) => void;
  loadMoreRef?: RefObject<HTMLTableRowElement | null>;
  isFetchingNextPage?: boolean;
  hasNextPage?: boolean;
}

export default function SummaryReportTable({
  data,
  isLoading,
  workUnitMinutes,
  sortBy,
  focusedRowIndex = -1,
  t,
  onSort,
  onEmployeeClick,
  loadMoreRef,
  isFetchingNextPage,
  hasNextPage,
}: SummaryReportTableProps) {
  useEffect(() => {
    if (focusedRowIndex >= 0) {
      if (focusedRowIndex === 0) {
        // Special case for first item: scroll the container to exactly top 0 
        // to avoid sticky header overlapping
        const row = document.getElementById(`summary-row-0`);
        const container = row?.closest('.overflow-auto');
        if (container) {
          container.scrollTo({ top: 0, behavior: 'smooth' });
        }
      } else {
        const row = document.getElementById(`summary-row-${focusedRowIndex}`);
        if (row) {
          row.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
      }
    }
  }, [focusedRowIndex]);

  const getSortIcon = (field: 'name' | 'work_hours' | 'leave_days') => {
    if (field === 'name') {
      if (sortBy === 'name_asc') return <ArrowUp size={12} className="ml-1 text-primary-600 dark:text-primary-400" />;
      if (sortBy === 'name_desc') return <ArrowDown size={12} className="ml-1 text-primary-600 dark:text-primary-400" />;
    }
    if (field === 'work_hours') {
      if (sortBy === 'work_hours_asc') return <ArrowUp size={12} className="ml-1 text-primary-600 dark:text-primary-400" />;
      if (sortBy === 'work_hours_desc') return <ArrowDown size={12} className="ml-1 text-primary-600 dark:text-primary-400" />;
    }
    if (field === 'leave_days') {
      if (sortBy === 'leave_days_asc') return <ArrowUp size={12} className="ml-1 text-primary-600 dark:text-primary-400" />;
      if (sortBy === 'leave_days_desc') return <ArrowDown size={12} className="ml-1 text-primary-600 dark:text-primary-400" />;
    }
    return <ArrowUpDown size={12} className="ml-1 opacity-20 group-hover:opacity-100 transition-opacity" />;
  };

  return (
    <table className="w-full text-left border-collapse">
      <thead>
        <tr className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky top-0 z-10">
          <th
            className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] cursor-pointer hover:text-primary-600 dark:text-primary-400 transition-colors group"
            onClick={() => onSort('name')}
          >
            <div className="flex items-center">
              {t("common.employee")}
              {getSortIcon('name')}
            </div>
          </th>

          <th
            className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right cursor-pointer hover:text-primary-600 dark:text-primary-400 transition-colors group"
            onClick={() => onSort('work_hours')}
          >
            <div className="flex items-center justify-end">
              {t("reports.table.std_hours")}
              {getSortIcon('work_hours')}
            </div>
          </th>

          <th className="px-8 py-6 text-[10px] font-bold text-orange-500 dark:text-orange-400 uppercase tracking-[0.25em] text-right">
            {t("reports.table.ot_hours")}
          </th>

          <th
            className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right cursor-pointer hover:text-primary-600 dark:text-primary-400 transition-colors group"
            onClick={() => onSort('leave_days')}
          >
            <div className="flex items-center justify-end">
              {t("reports.table.leave_days")}
              {getSortIcon('leave_days')}
            </div>
          </th>

          <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right">
            {t("common.notes")}
          </th>
          <th className="px-4 py-6 w-10"></th>
        </tr>
      </thead>

      <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
        {isLoading ? (
          Array.from({ length: 5 }).map((_, i) => (
            <tr key={`skeleton-${i}`} className="border-b border-slate-50 dark:border-slate-800/50">
              <td className="px-10 py-6">
                <div className="h-4 w-32 bg-slate-200 dark:bg-slate-800 rounded animate-pulse"></div>
              </td>
              <td className="px-8 py-6">
                <div className="flex flex-col items-end gap-1">
                  <div className="h-5 w-16 bg-slate-200 dark:bg-slate-800 rounded animate-pulse"></div>
                  <div className="h-2 w-8 bg-slate-100 dark:bg-slate-800/50 rounded animate-pulse"></div>
                </div>
              </td>
              <td className="px-8 py-6">
                <div className="flex flex-col items-end gap-1">
                  <div className="h-5 w-12 bg-orange-100 dark:bg-orange-900/20 rounded animate-pulse"></div>
                  <div className="h-2 w-8 bg-orange-50 dark:bg-orange-900/10 rounded animate-pulse"></div>
                </div>
              </td>
              <td className="px-10 py-6">
                <div className="flex justify-end">
                  <div className="h-5 w-16 bg-orange-100 dark:bg-orange-900/20 rounded animate-pulse"></div>
                </div>
              </td>
              <td className="px-10 py-6 text-right">—</td>
              <td className="px-4 py-6"></td>
            </tr>
          ))
        ) : data.length === 0 ? (
          <tr>
            <td colSpan={6} className="px-10 py-32">
              <div className="flex flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500 max-w-sm mx-auto">
                <div className="w-20 h-20 mb-6 rounded-3xl bg-slate-50 dark:bg-slate-800/80 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner border border-slate-100 dark:border-slate-700/50 rotate-3">
                  <SearchX size={32} className="-rotate-3" />
                </div>
                <h3 className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest mb-2">
                  {t("reports.messages.no_data")}
                </h3>
                <p className="text-xs font-medium text-slate-500 dark:text-slate-400">
                  Try adjusting the date range or employee filter to find what you're looking for.
                </p>
              </div>
            </td>
          </tr>
        ) : (
          data.map((row: AttendanceSummary, index: number) => (
            <tr
              key={row.employee_id}
              id={`summary-row-${index}`}
              onClick={() => onEmployeeClick(row.employee_id, row.employee_name)}
              className={cn(
                "hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group cursor-pointer",
                focusedRowIndex === index ? "bg-primary-50/50 dark:bg-primary-900/20 ring-1 ring-inset ring-primary-500/50" : ""
              )}
            >
              <td className="px-10 py-6 font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">
                {row.employee_name}
              </td>

              <td className="px-8 py-6 text-right">
                <div className="font-mono font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter cursor-default" title={`${row.total_std_units} units`}>
                  {formatUnitsToHours(row.total_std_units, workUnitMinutes)}h
                </div>
              </td>

              <td className="px-8 py-6 text-right">
                {row.total_ot_units > 0 ? (
                  <div className="font-mono font-bold text-orange-600 dark:text-orange-400 text-base tracking-tighter cursor-default" title={`${row.total_ot_units} OT units`}>
                    {formatUnitsToHours(row.total_ot_units, workUnitMinutes)}h
                  </div>
                ) : (
                  <span className="text-slate-200 dark:text-slate-700 font-bold">—</span>
                )}
              </td>

              <td className="px-10 py-6 text-right font-mono font-bold text-orange-600 dark:text-orange-300 text-base tracking-tighter">
                {row.total_leave_days} {t("common.days_short")}
              </td>

              <td className="px-10 py-6 text-right text-slate-200 dark:text-slate-700 font-bold tracking-widest">—</td>

              <td className="px-4 py-6 text-right">
                <ChevronRight size={16} className="text-slate-300 dark:text-slate-600 group-hover:text-primary-500 dark:group-hover:text-primary-400 transition-colors group-hover:translate-x-1 transform duration-200" />
              </td>
            </tr>
          ))
        )}
        {/* Sentinel row for infinite scroll */}
        {loadMoreRef && (
          <tr ref={loadMoreRef}>
            <td colSpan={6} className="p-0">
              {isFetchingNextPage && (
                <div className="flex items-center justify-center py-6">
                  <Loader2 size={20} className="animate-spin text-primary-500" />
                  <span className="ml-2 text-xs font-bold text-slate-400 uppercase tracking-widest">{t("common.loading")}</span>
                </div>
              )}
            </td>
          </tr>
        )}
      </tbody>
    </table>
  );
}
