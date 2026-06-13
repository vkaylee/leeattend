import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { NavigableListColumn } from "@/components/common/navigable-list";
import { AttendanceSummary, AttendanceReportSort } from "@/lib/api/types";
import { formatHours } from "@/lib/utils/attendance";

interface GetReportColumnsProps {
  handleSort: (field: "name" | "work_hours" | "leave_days" | "ot_hours" | "total_hours") => void;
  sortBy: AttendanceReportSort | undefined;
  t: TFunction;
}

export const getReportColumns = ({
  handleSort,
  sortBy,
  t,
}: GetReportColumnsProps): NavigableListColumn<AttendanceSummary>[] => {
  const getSortIcon = (field: "name" | "work_hours" | "leave_days" | "ot_hours" | "total_hours") => {
    if (sortBy === `${field}_asc`) return <ArrowUp className="ml-1 text-primary-600 dark:text-primary-400" size={12} />;
    if (sortBy === `${field}_desc`) return <ArrowDown className="ml-1 text-primary-600 dark:text-primary-400" size={12} />;
    return <ArrowUpDown className="ml-1 opacity-20 group-hover:opacity-100 transition-opacity" size={12} />;
  };

  return [
    {
      render: (row) => (
        <div>
          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{row.employee_name}</div>
          <div className="flex items-center space-x-2 mt-0.5">
            <span className="text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest leading-none">{row.employee_id_number ?? "—"}</span>
            {row.employer_name && (
              <>
                <span className="text-[9px] text-slate-300 dark:text-slate-600 leading-none">•</span>
                <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wide truncate max-w-[120px] leading-none">{row.employer_name}</span>
              </>
            )}
          </div>
        </div>
      ),
      cellClassName: "px-10 py-6",
      header: <div className="flex items-center">{t("common.employee")}{getSortIcon("name")}</div>,
      headerClassName: "px-10",
      key: "name",
      onHeaderClick: () => { handleSort("name"); },
    },
    {
      render: (row) => (
        <div className="font-mono font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter cursor-default" title={`${row.total_std_units} units`}>
          {formatHours(row.computed_std_hours)}h
        </div>
      ),
      cellClassName: "py-6 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.std_hours")}{getSortIcon("work_hours")}</div>,
      headerClassName: "text-right",
      key: "std_hours",
      onHeaderClick: () => { handleSort("work_hours"); },
    },
    {
      render: (row) => row.total_ot_units > 0 ? (
        <div className="font-mono font-bold text-orange-600 dark:text-orange-400 text-base tracking-tighter cursor-default" title={`${row.total_ot_units} OT units`}>
          {formatHours(row.computed_ot_hours)}h
        </div>
      ) : (
        <span className="text-slate-200 dark:text-slate-700 font-bold">—</span>
      ),
      cellClassName: "py-6 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.ot_hours")}{getSortIcon("ot_hours")}</div>,
      headerClassName: "text-right text-orange-500 dark:text-orange-400",
      key: "ot_hours",
      onHeaderClick: () => { handleSort("ot_hours"); },
    },
    {
      render: (row) => (
        <div className="font-mono font-bold text-slate-700 dark:text-slate-200 text-base tracking-tighter cursor-default" title={`${row.total_std_units + row.total_ot_units} total units`}>
          {formatHours(row.computed_std_hours + row.computed_ot_hours)}h
        </div>
      ),
      cellClassName: "py-6 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.total_hours")}{getSortIcon("total_hours")}</div>,
      headerClassName: "text-right",
      key: "total_hours",
      onHeaderClick: () => { handleSort("total_hours"); },
    },
    {
      render: (row) => <span className="font-mono font-bold text-orange-600 dark:text-orange-300 text-base tracking-tighter">{row.total_leave_days} {t("common.days_short")}</span>,
      cellClassName: "px-10 py-6 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.leave_days")}{getSortIcon("leave_days")}</div>,
      headerClassName: "px-10 text-right",
      key: "leave_days",
      onHeaderClick: () => { handleSort("leave_days"); },
    },
    {
      render: () => <span className="text-slate-200 dark:text-slate-700 font-bold tracking-widest">—</span>,
      cellClassName: "px-10 py-6 text-right",
      header: t("common.notes"),
      headerClassName: "px-10 text-right",
      key: "notes",
    },
  ];
};
