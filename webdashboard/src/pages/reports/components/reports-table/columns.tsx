import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { EmployeeIdentityCell } from "@/components/common/employee-identity-cell";
import { NavigableListColumn } from "@/components/common/navigable-list";
import { AttendanceSummary, AttendanceReportSort } from "@/lib/api/types";
import { cn } from "@/lib/utils";
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
        <EmployeeIdentityCell
          avatarClassName="h-9 w-9 text-xs"
          code={row.employee_id_number}
          name={row.employee_name}
          subtitle={row.employer_name}
        />
      ),
      cellClassName: "px-6 py-4 min-w-[260px]",
      header: <div className="flex items-center">{t("reports.table.employee", "Nhân viên")}{getSortIcon("name")}</div>,
      headerClassName: "px-6 min-w-[260px]",
      key: "name",
      onHeaderClick: () => { handleSort("name"); },
    },
    {
      render: (row) => (
        <div className="font-mono font-semibold text-slate-800 dark:text-slate-100 text-sm tracking-tight cursor-default" title={`${row.total_std_units} units`}>
          {formatHours(row.computed_std_hours)}h
        </div>
      ),
      cellClassName: "py-4 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.std_hours")}{getSortIcon("work_hours")}</div>,
      headerClassName: "text-right",
      key: "std_hours",
      onHeaderClick: () => { handleSort("work_hours"); },
    },
    {
      render: (row) => row.total_ot_units > 0 ? (
        <div className="font-mono font-semibold text-amber-600 dark:text-amber-400 text-sm tracking-tight cursor-default" title={`${row.total_ot_units} OT units`}>
          {formatHours(row.computed_ot_hours)}h
        </div>
      ) : (
        <span className="text-slate-300 dark:text-slate-600 font-normal">—</span>
      ),
      cellClassName: "py-4 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.ot_hours")}{getSortIcon("ot_hours")}</div>,
      headerClassName: "text-right",
      key: "ot_hours",
      onHeaderClick: () => { handleSort("ot_hours"); },
    },
    {
      render: (row) => (
        <div className="font-mono font-semibold text-slate-900 dark:text-white text-sm tracking-tight cursor-default" title={`${row.total_std_units + row.total_ot_units} total units`}>
          {formatHours(row.computed_std_hours + row.computed_ot_hours)}h
        </div>
      ),
      cellClassName: "py-4 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.total_hours")}{getSortIcon("total_hours")}</div>,
      headerClassName: "text-right",
      key: "total_hours",
      onHeaderClick: () => { handleSort("total_hours"); },
    },
    {
      render: (row) => (
        <span
          className={cn(
            "font-mono text-sm tracking-tight",
            row.total_leave_days > 0
              ? "font-semibold text-primary-600 dark:text-primary-400"
              : "font-normal text-slate-400 dark:text-slate-500"
          )}
        >
          {row.total_leave_days} {t("common.days_short", "ngày")}
        </span>
      ),
      cellClassName: "px-6 py-4 text-right",
      header: <div className="flex items-center justify-end">{t("reports.table.leave_days")}{getSortIcon("leave_days")}</div>,
      headerClassName: "px-6 text-right",
      key: "leave_days",
      onHeaderClick: () => { handleSort("leave_days"); },
    },
    {
      render: () => <span className="text-slate-200 dark:text-slate-700 font-normal">—</span>,
      cellClassName: "px-6 py-4 text-right",
      header: t("reports.table.notes", "Ghi chú"),
      headerClassName: "px-6 text-right",
      key: "notes",
    },
  ];
};
