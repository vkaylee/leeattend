import { Building2, ChevronRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import NavigableList from "@/components/common/navigable-list";
import { CustomFieldsFilterPopover } from "@/components/custom-fields/custom-fields-filter-popover";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";
import StatusBadge from "@/components/ui/status-badge";
import { cn } from "@/lib/utils";

import type { Employee, Department } from "../../../lib/api/types";
import type { SortKey, SortDir } from "../hooks/use-employees-page";
import type { EmployeeModalState } from "../types/employee-modals";

import { useEmployeeColumns } from "../hooks/use-employee-columns";

function EmployeeMobileCard({ employee, isSelected, onSelect }: {
  employee: Employee;
  isSelected: boolean;
  onSelect: () => void;
}) {
  const { t } = useTranslation();

  return (
    <Button
      className={cn(
        "!h-auto w-full min-w-0 flex-col items-stretch justify-start rounded-2xl border bg-white p-5 text-left shadow-sm transition-all active:scale-[0.99] dark:bg-slate-900",
        isSelected
          ? "border-primary-300 ring-2 ring-primary-500/20 dark:border-primary-700"
          : "border-slate-100 dark:border-slate-800",
      )}
      onClick={onSelect}
      type="button"
      variant="ghost"
    >
      <div className="flex min-w-0 items-start gap-3">
        <div className={cn(
          "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border font-bold uppercase",
          isSelected
            ? "border-primary-500 bg-primary-600 text-white"
            : "border-slate-100 bg-slate-100 text-slate-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300",
        )}>
          {employee.full_name[0]}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">{employee.full_name}</div>
          <div className="mt-1 flex items-center gap-2">
            <span className="font-mono text-[10px] font-bold uppercase tracking-wider text-slate-400">{employee.employee_id_number}</span>
            {!!employee.job_title && (
              <span className="truncate text-[10px] font-semibold text-slate-500 dark:text-slate-400">
                • {employee.job_title}
              </span>
            )}
          </div>
        </div>
        <ChevronRight className="mt-1 h-4 w-4 shrink-0 text-slate-300 dark:text-slate-600" />
      </div>
      <div className="mt-4 flex min-w-0 flex-wrap items-center justify-between gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        {employee.department_name ? (
          <span className="inline-flex min-w-0 items-center gap-1.5 rounded-xl border border-blue-100 bg-blue-50 px-3 py-1.5 text-[10px] font-bold uppercase tracking-wider text-blue-600 dark:border-blue-800 dark:bg-blue-950 dark:text-blue-300">
            <Building2 size={11} /><span className="truncate">{employee.department_name}</span>
          </span>
        ) : <span className="text-xs text-slate-300 dark:text-slate-600">—</span>}
        <StatusBadge variant={employee.active_employment_id ? "success" : "neutral"}>
          {employee.active_employment_id ? t("employees.table.active") : t("employees.table.terminated")}
        </StatusBadge>
      </div>
    </Button>
  );
}

interface EmployeeTableProps {
  customFieldFilters: Record<string, string>;
  departments: Department[] | undefined;
  employees: Employee[];
  fetchNextPage: () => void;
  filterDeptIds: string[];
  filterStatus: string;
  formatDate?: (date: string) => string;
  hasNextPage: boolean | undefined;
  isAllSelected?: boolean;
  isFetching: boolean;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  isPlaceholderData: boolean;
  onOpenModal?: (modal: EmployeeModalState) => void;
  onToggleSelectAll?: () => void;
  onToggleSelectRow?: (id: string) => void;
  searchInputRef: React.RefObject<HTMLInputElement | null>;
  searchTerm: string;
  selectedEmployee: Employee | null;
  selectedRowIds?: string[];
  setCustomFieldFilters: (filters: Record<string, string>) => void;
  setFilterDeptIds: (ids: string[]) => void;
  setFilterStatus: (status: string) => void;
  setSearchTerm: (s: string) => void;
  setSelectedEmployee: (emp: Employee) => void;
  sortDir: SortDir;
  sortKey: SortKey | null;
  toggleDeptFilter: (id: string) => void;
  toggleSort: (key: SortKey) => void;
  totalItems: number;
}

export function EmployeeTable(props: EmployeeTableProps) {
  const { t } = useTranslation();
  const {
    customFieldFilters,
    departments,
    employees,
    fetchNextPage,
    filterDeptIds,
    filterStatus,
    formatDate,
    hasNextPage,
    isAllSelected,
    isFetching,
    isFetchingNextPage,
    isLoading,
    isPlaceholderData,
    onOpenModal,
    onToggleSelectAll,
    onToggleSelectRow,
    searchInputRef,
    searchTerm,
    selectedEmployee,
    selectedRowIds,
    setCustomFieldFilters,
    setFilterDeptIds,
    setFilterStatus,
    setSearchTerm,
    setSelectedEmployee,
    sortDir,
    sortKey,
    toggleDeptFilter,
    toggleSort,
  } = props;

  const employeeColumns = useEmployeeColumns({
    departments,
    filterDeptIds,
    filterStatus,
    formatDate,
    isAllSelected,
    onOpenModal,
    onSelectEmployee: setSelectedEmployee,
    onToggleSelectAll,
    onToggleSelectRow,
    selectedEmployee,
    selectedRowIds,
    setFilterDeptIds,
    setFilterStatus,
    sortDir,
    sortKey,
    toggleDeptFilter,
    toggleSort,
  });
  const departmentOptions = [
    { label: t("common.all"), value: "all" },
    ...(departments ?? []).map((department) => ({ label: department.name, value: department.id })),
  ];
  const statusOptions = [
    { label: t("employees.table.active"), value: "active" },
    { label: t("employees.table.terminated"), value: "terminated" },
    { label: t("common.all"), value: "all" },
  ];

  return (
    <div className={cn("transition-opacity duration-300", isPlaceholderData && isFetching && "opacity-50")}>
      <div className="mb-4 flex items-center gap-2 lg:hidden">
        <ResponsiveToolbar
          filterCount={filterDeptIds.length + (filterStatus !== "active" ? 1 : 0) + Object.keys(customFieldFilters).length}
          mobileOnly
        >
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("common.department")}</label>
              <Combobox
                onChange={(value) => { setFilterDeptIds(value && value !== "all" ? [value] : []); }}
                options={departmentOptions}
                value={filterDeptIds[0] ?? "all"}
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("employees.table.status")}</label>
              <Combobox onChange={(value) => { setFilterStatus(value !== "" ? value : "active"); }} options={statusOptions} value={filterStatus} />
            </div>
            <CustomFieldsFilterPopover customFieldFilters={customFieldFilters} setCustomFieldFilters={setCustomFieldFilters} />
          </div>
        </ResponsiveToolbar>
      </div>
      <NavigableList<Employee>
        className="animate-in slide-in-from-bottom-8 duration-700"
        columns={employeeColumns}
        emptyMessage={t("employees.table.no_data")}
        fetchNextPage={fetchNextPage}
        hasNextPage={hasNextPage}
        headerSlot={
          <CustomFieldsFilterPopover
            customFieldFilters={customFieldFilters}
            setCustomFieldFilters={setCustomFieldFilters}
          />
        }
        isFetchingNextPage={isFetchingNextPage}
        isLoading={isLoading}
        itemKey={(emp) => emp.id}
        items={employees}
        maxHeightClass=""
        onSearchChange={setSearchTerm}
        onSelect={setSelectedEmployee}
        renderMobileCard={(employee) => (
          <EmployeeMobileCard
            employee={employee}
            isSelected={selectedEmployee?.id === employee.id}
            onSelect={() => { setSelectedEmployee(employee); }}
          />
        )}
        resetDeps={[searchTerm, filterDeptIds, sortKey, sortDir, customFieldFilters]}
        rowIdPrefix="employee-row"
        searchInputRef={searchInputRef}
        searchPlaceholder={t("employees.search_placeholder")}
        searchTestId="employee-search"
        searchValue={searchTerm}
        selectedKey={selectedEmployee?.id}
        showChevron
        testIdPrefix="employees-row"
      />
    </div>
  );
}
