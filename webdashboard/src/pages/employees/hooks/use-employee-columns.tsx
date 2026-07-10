import { ArrowUpDown, Building2, ChevronDown, ChevronUp } from "lucide-react";
import React, { useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { NavigableListColumn } from "@/components/common/navigable-list";

import StatusBadge from "@/components/ui/status-badge";

import type { Employee, Department } from "../../../lib/api/types";
import type { SortKey, SortDir } from "./use-employees-page";

import { DepartmentFilterHeader, EmployeeNameCell, StatusFilterHeader } from "../components/employee-table-subcomponents";

interface UseEmployeeColumnsProps {
  departments: Department[] | undefined;
  filterDeptIds: string[];
  filterStatus: string;
  selectedEmployee: Employee | null;
  setFilterDeptIds: (ids: string[]) => void;
  setFilterStatus: (status: string) => void;
  sortDir: SortDir;
  sortKey: SortKey | null;
  toggleDeptFilter: (id: string) => void;
  toggleSort: (key: SortKey) => void;
}

export const useEmployeeColumns = ({
  departments,
  filterDeptIds,
  filterStatus,
  selectedEmployee,
  setFilterDeptIds,
  setFilterStatus,
  sortDir,
  sortKey,
  toggleDeptFilter,
  toggleSort,
}: UseEmployeeColumnsProps) => {
  const { t } = useTranslation();

  const renderSortIcon = useCallback((col: SortKey) => {
    if (sortKey !== col)
      return <ArrowUpDown className="opacity-0 group-hover:opacity-50 transition-opacity ml-1" size={10} />;
    return sortDir === "asc" ? (
      <ChevronUp className="text-primary-600 dark:text-primary-400 ml-1" size={10} />
    ) : (
      <ChevronDown className="text-primary-600 dark:text-primary-400 ml-1" size={10} />
    );
  }, [sortKey, sortDir]);

  const columns: NavigableListColumn<Employee>[] = useMemo(() => [
    {
      render: (emp) => (
        <EmployeeNameCell employee={emp} isSelected={selectedEmployee?.id === emp.id} />
      ),
      cellClassName: "py-6",
      header: <span className="inline-flex items-center">{t("employees.table.name")}{renderSortIcon("name")}</span>,
      key: "name",
      onHeaderClick: () => { toggleSort("name"); },
    },
    {
      render: (emp) => (
        <span className="font-mono font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-tight">
          {emp.employee_id_number}
        </span>
      ),
      cellClassName: "py-6",
      header: <span className="inline-flex items-center">{t("employees.form.employee_id")}{renderSortIcon("id")}</span>,
      key: "id",
      onHeaderClick: () => { toggleSort("id"); },
    },
    {
      render: (emp) => emp.department_name ? (
        <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800">
          <Building2 className="opacity-60" size={12} />{emp.department_name}
        </span>
      ) : <span className="text-slate-300 dark:text-slate-600">—</span>,
      cellClassName: "py-6",
      header: (
        <span className="inline-flex items-center gap-2">
          <span className="inline-flex items-center">
            {t("common.department")}{renderSortIcon("department")}
          </span>
          <DepartmentFilterHeader 
            departments={departments}
            filterDeptIds={filterDeptIds}
            setFilterDeptIds={setFilterDeptIds}
            toggleDeptFilter={toggleDeptFilter}
          />
        </span>
      ),
      key: "department",
      onHeaderClick: () => { toggleSort("department"); },
    },
    {
      render: (emp) => (
        <StatusBadge variant={emp.active_employment_id ? "success" : "neutral"}>
          {emp.active_employment_id ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}
        </StatusBadge>
      ),
      cellClassName: "py-6 text-center",
      header: (
        <span className="inline-flex items-center gap-2">
          <span>{t("employees.table.status")}</span>
          <StatusFilterHeader
            filterStatus={filterStatus}
            setFilterStatus={setFilterStatus}
          />
        </span>
      ),
      headerClassName: "text-center",
      key: "status",
    },
  ], [
    renderSortIcon,
    t,
    filterDeptIds,
    filterStatus,
    departments,
    selectedEmployee,
    toggleSort,
    setFilterDeptIds,
    setFilterStatus,
    toggleDeptFilter
  ]);

  return columns;
};
