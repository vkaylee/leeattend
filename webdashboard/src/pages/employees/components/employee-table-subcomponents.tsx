import { Building2, ListFilter, UserCheck, UserX, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import { cn } from "@/lib/utils";

import type { Employee, Department } from "../../../lib/api/types";

interface EmployeeNameCellProps {
  employee: Employee;
  isSelected: boolean;
}

export const EmployeeNameCell = ({ employee, isSelected }: EmployeeNameCellProps) => (
  <div className="flex items-center gap-5">
    <div className={cn(
      "w-12 h-12 rounded-2xl flex items-center justify-center font-bold text-sm shadow-inner border transition-all uppercase",
      isSelected 
        ? "bg-primary-600 text-white border-primary-500" 
        : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100"
    )}>
      {employee.full_name[0]}
    </div>
    <div>
      <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{employee.full_name}</div>
      <div className="font-mono font-bold text-[10px] text-slate-400 dark:text-slate-500 uppercase tracking-tighter mt-1.5">{employee.employee_id_number}</div>
    </div>
  </div>
);

interface StatusFilterHeaderProps {
  filterStatus: string;
  setFilterStatus: (status: string) => void;
}

export function StatusFilterHeader({
  filterStatus,
  setFilterStatus,
}: StatusFilterHeaderProps) {
  const { t } = useTranslation();

  const statusOptions = [
    { value: "active", label: t("employees.table.active", "Hoạt động"), icon: <UserCheck size={12} /> },
    { value: "terminated", label: t("employees.table.terminated", "Đã nghỉ việc"), icon: <UserX size={12} /> },
    { value: "all", label: t("common.all", "Tất cả"), icon: <Users size={12} /> },
  ];

  return (
    <FilterPanel
      container="popover"
      data-testid="status-filter"
      mode="single"
      options={statusOptions}
      selected={filterStatus !== "active" ? [filterStatus] : []}
      onToggle={(val) => setFilterStatus(val)}
      onClear={() => setFilterStatus("active")}
      headerLabel={t("employees.filter_status", "Filter by status")}
      trigger={
        <FilterTrigger
          variant="icon-button"
          label={t("employees.filter_status", "Filter by status")}
          hasActive={filterStatus !== "active"}
          icon={<ListFilter size={12} />}
          data-testid="status-filter-btn"
        />
      }
    />
  );
}

interface DepartmentFilterHeaderProps {
  departments: Department[] | undefined;
  filterDeptIds: string[];
  setFilterDeptIds: (ids: string[]) => void;
  toggleDeptFilter: (id: string) => void;
}

export function DepartmentFilterHeader({
  departments,
  filterDeptIds,
  setFilterDeptIds,
  toggleDeptFilter,
}: DepartmentFilterHeaderProps) {
  const { t } = useTranslation();
  const deptOptions = (departments ?? []).map(d => ({
    value: d.id,
    label: d.name,
    icon: <Building2 className="opacity-50 shrink-0" size={12} />,
  }));

  return (
    <FilterPanel
      container="popover"
      mode="multi"
      options={deptOptions}
      selected={filterDeptIds}
      onToggle={toggleDeptFilter}
      onClear={() => setFilterDeptIds([])}
      headerLabel={t("employees.filter_department", "Filter by department")}
      emptyMessage={t("employees.no_departments", "Chưa có phòng ban")}
      data-testid="dept-filter"
      trigger={
        <FilterTrigger
          variant="icon-button"
          label={t("employees.filter_department", "Filter by department")}
          activeCount={filterDeptIds.length}
          hasActive={filterDeptIds.length > 0}
          icon={<ListFilter size={12} />}
          data-testid="dept-filter-btn"
        />
      }
    />
  );
}

