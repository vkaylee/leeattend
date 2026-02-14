import { ArrowUp, ArrowDown, ArrowUpDown } from "lucide-react";
import { AttendanceSummary, AttendanceReportSort } from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";

interface SummaryReportTableProps {
  data: AttendanceSummary[];
  isLoading: boolean;
  workUnitMinutes: number;
  sortBy: AttendanceReportSort | undefined;
  t: (key: string) => string;
  onSort: (field: 'name' | 'work_hours' | 'leave_days') => void;
}

export default function SummaryReportTable({
  data,
  isLoading,
  workUnitMinutes,
  sortBy,
  t,
  onSort,
}: SummaryReportTableProps) {
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
        <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
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
            className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right cursor-pointer hover:text-primary-600 dark:text-primary-400 transition-colors group"
            onClick={() => onSort('work_hours')}
          >
            <div className="flex items-center justify-end">
              {t("reports.table.total_hours")}
              {getSortIcon('work_hours')}
            </div>
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
        </tr>
      </thead>

      <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
        {isLoading ? (
          <tr>
            <td colSpan={4} className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">
              {t("reports.messages.generating")}
            </td>
          </tr>
        ) : data.length === 0 ? (
          <tr>
            <td colSpan={4} className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
              {t("reports.messages.no_data")}
            </td>
          </tr>
        ) : (
          data.map((row: AttendanceSummary) => (
            <tr key={row.employee_id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
              <td className="px-10 py-6 font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">
                {row.employee_name}
              </td>

              <td className="px-10 py-6 text-right">
                <div className="font-mono font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter">
                  {formatUnitsToHours(row.total_work_units, workUnitMinutes)}h
                </div>
                <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-60">
                  {row.total_work_units} UNITS
                </span>
              </td>

              <td className="px-10 py-6 text-right font-mono font-bold text-orange-600 dark:text-orange-300 text-base tracking-tighter">
                {row.total_leave_days} {t("common.days_short")}
              </td>

              <td className="px-10 py-6 text-right text-slate-200 dark:text-slate-700 font-bold tracking-widest">—</td>
            </tr>
          ))
        )}
      </tbody>
    </table>
  );
}
