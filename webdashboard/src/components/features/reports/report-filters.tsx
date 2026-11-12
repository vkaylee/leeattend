import { Calendar, Building2, Briefcase } from "lucide-react";
import React from "react";

import type { Department, Employer } from "@/lib/api/types";

import { FilterModeButtons } from "@/components/common/filter-mode-buttons";
import Button from "@/components/ui/button";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import {
  SEGMENTED_CONTROL_STYLE,
  SEGMENTED_ITEM_STYLE,
} from "@/components/ui/control-styles";
import { SearchInput } from "@/components/ui/search-input";
import ToolbarRow from "@/components/ui/toolbar-row";
import { cn } from "@/lib/utils";

interface ReportFiltersProps {
  dateRange: { start: string; end: string };
  departments?: Department[];
  employeeStatus: string;
  employers?: Employer[];
  filterMode: "this_month" | "last_month" | "custom" | "specific_month";
  onDateRangeChange: (range: { start: string; end: string }) => void;
  onDepartmentChange: (departmentId: string | null) => void;
  onEmployeeStatusChange: (status: string) => void;
  onEmployerChange?: (employerId: string | null) => void;
  onFilterModeChange: (mode: "this_month" | "last_month" | "custom" | "specific_month") => void;
  onSearchKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  onSearchQueryChange: (query: string) => void;
  onSpecificMonthChange: (month: number, year: number) => void;
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  searchQuery: string;
  selectedDepartmentId: string | null;
  selectedEmployerId?: string | null;
  specificMonth: { month: number; year: number };
  t: (key: string, fallback?: string) => string;
}

function StatusFilterSelector({
  employeeStatus,
  onEmployeeStatusChange,
  t,
}: {
  employeeStatus: string;
  onEmployeeStatusChange: (status: string) => void;
  t: ReportFiltersProps["t"];
}) {
  const options = ["active", "terminated", "all"] as const;
  return (
    <div className={SEGMENTED_CONTROL_STYLE}>
      {options.map(status => (
        <Button
          className={cn(
            SEGMENTED_ITEM_STYLE,
            employeeStatus === status
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )}
          key={status}
          onClick={() => { onEmployeeStatusChange(status); }}
          size="sm"
          variant="ghost"
        >
          {t(`reports.filters.status.${status}`)}
        </Button>
      ))}
    </div>
  );
}

function DepartmentChips({
  departments,
  onDepartmentChange,
  selectedDepartmentId,
  t,
}: {
  departments: Department[];
  onDepartmentChange: ReportFiltersProps["onDepartmentChange"];
  selectedDepartmentId: ReportFiltersProps["selectedDepartmentId"];
  t: ReportFiltersProps["t"];
}) {
  return (
    <div className="flex flex-wrap items-center gap-1.5 pt-1" data-testid="department-chips">
      <Building2 className="text-slate-400 dark:text-slate-500 shrink-0" size={14} />
      <Button
        className={cn(
          "px-3 py-1 rounded-full text-xs font-semibold tracking-wide transition-all border h-auto",
          selectedDepartmentId === null
            ? "bg-primary-600 dark:bg-primary-500 text-white border-primary-600 dark:border-primary-500 shadow-sm hover:bg-primary-700 dark:hover:bg-primary-600 hover:text-white"
            : "bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
        )}
        onClick={() => { onDepartmentChange(null); }}
        variant="ghost"
      >
        {t("reports.all_departments", "Tất cả")}
      </Button>
      {departments.map((dept) => (
        <Button
          className={cn(
            "px-3 py-1 rounded-full text-xs font-semibold tracking-wide transition-all border h-auto",
            selectedDepartmentId === dept.id
              ? "bg-primary-600 dark:bg-primary-500 text-white border-primary-600 dark:border-primary-500 shadow-sm hover:bg-primary-700 dark:hover:bg-primary-600 hover:text-white"
              : "bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
          )}
          key={dept.id}
          onClick={() => { onDepartmentChange(dept.id); }}
          variant="ghost"
        >
          {dept.name}
        </Button>
      ))}
    </div>
  );
}

function EmployerFilterSelector({
  employers,
  selectedEmployerId,
  onEmployerChange,
  t,
}: {
  employers?: Employer[];
  selectedEmployerId?: string | null;
  onEmployerChange?: (employerId: string | null) => void;
  t: ReportFiltersProps["t"];
}) {
  if (!employers || employers.length === 0) return null;

  const selectedEmployer = employers.find(emp => emp.id === selectedEmployerId);

  return (
    <FilterPanel
      container="dropdown"
      mode="single"
      onClear={() => onEmployerChange?.(null)}
      onToggle={(val) => onEmployerChange?.(val === "__all__" ? null : val)}
      options={[
        { value: "__all__", label: t("reports.all_employers") },
        ...employers.map(e => ({ value: e.id, label: e.name })),
      ]}
      selected={selectedEmployerId ? [selectedEmployerId] : []}
      trigger={
        <FilterTrigger
          hasActive={!!selectedEmployerId}
          icon={<Briefcase size={14} />}
          label={selectedEmployer ? selectedEmployer.name : t("reports.all_employers")}
          size="sm"
          variant="pill-button"
        />
      }
    />
  );
}

export default function ReportFilters({
  dateRange,
  departments,
  employers,
  selectedEmployerId,
  onEmployerChange,
  filterMode,
  onDateRangeChange,
  onDepartmentChange,
  onFilterModeChange,
  onSpecificMonthChange,
  selectedDepartmentId,
  specificMonth,
  t,
  employeeStatus,
  onEmployeeStatusChange,
  searchQuery,
  onSearchQueryChange,
  searchInputRef,
  onSearchKeyDown,
}: ReportFiltersProps) {
  return (
    <div className="flex flex-col gap-3">
      <ToolbarRow className="lg:flex-nowrap lg:justify-between">
        <div className="flex min-w-0 flex-1 flex-nowrap items-center gap-2 overflow-x-auto">
          <FilterModeButtons
            filterMode={filterMode}
            onFilterModeChange={onFilterModeChange}
            onSpecificMonthChange={onSpecificMonthChange}
            size="sm"
            specificMonth={specificMonth}
            t={t}
          />
          <div className="hidden h-6 w-px shrink-0 bg-slate-200 sm:block dark:bg-slate-800" />
          <StatusFilterSelector employeeStatus={employeeStatus} onEmployeeStatusChange={onEmployeeStatusChange} t={t} />
          {!!employers && employers.length > 0 && (
            <>
              <div className="hidden h-6 w-px shrink-0 bg-slate-200 sm:block dark:bg-slate-800" />
              <EmployerFilterSelector employers={employers} onEmployerChange={onEmployerChange} selectedEmployerId={selectedEmployerId} t={t} />
            </>
          )}
        </div>

        <div className="min-w-[220px] max-w-sm shrink-0 sm:w-auto">
          <SearchInput
            containerClassName="bg-white dark:bg-slate-900"
            data-testid="report-search-shared"
            onChange={(e) => onSearchQueryChange(e.target.value)}
            onClear={() => onSearchQueryChange("")}
            onFocus={(e) => e.currentTarget.select()}
            onKeyDown={onSearchKeyDown}
            placeholder={t("reports.search_placeholder", "Tìm theo tên nhân viên...")}
            ref={searchInputRef}
            size="sm"
            value={searchQuery}
          />
        </div>
      </ToolbarRow>

      {departments && departments.length > 0 ? (
        <DepartmentChips departments={departments} onDepartmentChange={onDepartmentChange} selectedDepartmentId={selectedDepartmentId} t={t} />
      ) : null}

      {filterMode === "custom" ? (
        <div className="bg-white dark:bg-slate-900 p-5 rounded-2xl border border-slate-200/80 dark:border-slate-800 shadow-sm grid grid-cols-1 sm:grid-cols-2 gap-4 animate-in slide-in-from-top-2 duration-300 max-w-xl">
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-slate-600 dark:text-slate-400 flex items-center gap-1.5">
              <Calendar size={13} /> {t("common.from_date", "Từ ngày")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 outline-none focus:ring-2 focus:ring-primary-500/20 focus:bg-white dark:focus:bg-slate-900 transition-all font-medium text-xs text-slate-800 dark:text-slate-100"
              onChange={(e) => { onDateRangeChange({ ...dateRange, start: e.target.value }); }}
              type="date"
              value={dateRange.start}
            />
          </div>
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-slate-600 dark:text-slate-400 flex items-center gap-1.5">
              <Calendar size={13} /> {t("common.to_date", "Đến ngày")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 outline-none focus:ring-2 focus:ring-primary-500/20 focus:bg-white dark:focus:bg-slate-900 transition-all font-medium text-xs text-slate-800 dark:text-slate-100"
              onChange={(e) => { onDateRangeChange({ ...dateRange, end: e.target.value }); }}
              type="date"
              value={dateRange.end}
            />
          </div>
        </div>
      ) : null}
    </div>
  );
}
