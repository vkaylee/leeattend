import { Building2, Calendar, Briefcase, RotateCcw } from "lucide-react";
import React from "react";

import type { Department, Employer } from "@/lib/api/types";

import { FilterModeButtons } from "@/components/common/filter-mode-buttons";
import Button from "@/components/ui/button";
import {
  SEGMENTED_CONTROL_STYLE,
  SEGMENTED_ITEM_STYLE,
} from "@/components/ui/control-styles";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import { SearchInput } from "@/components/ui/search-input";
import { ToolbarRow } from "@/components/ui/toolbar-row";
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
    <div aria-label={t("common.filter_by_status", "Trạng thái nhân viên")} className={SEGMENTED_CONTROL_STYLE} role="group">
      {options.map((status) => (
        <Button
          aria-pressed={employeeStatus === status}
          className={cn(
            SEGMENTED_ITEM_STYLE,
            employeeStatus === status
              ? "bg-white text-primary-600 shadow-sm dark:bg-slate-900 dark:text-primary-400"
              : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200",
          )}
          key={status}
          onClick={() => { onEmployeeStatusChange(status); }}
          size="sm"
          type="button"
          variant="ghost"
        >
          {t(`reports.filters.status.${status}`)}
        </Button>
      ))}
    </div>
  );
}

function DepartmentFilter({
  departments,
  onDepartmentChange,
  selectedDepartmentId,
  t,
}: {
  departments: Department[];
  onDepartmentChange: ReportFiltersProps["onDepartmentChange"];
  selectedDepartmentId: string | null;
  t: ReportFiltersProps["t"];
}) {
  const selectedDepartment = departments.find((department) => department.id === selectedDepartmentId);

  return (
    <FilterPanel
      container="popover"
      data-testid="department-filter"
      headerLabel={t("reports.department_filter_label", "Phòng ban")}
      mode="single"
      onClear={() => { onDepartmentChange(null); }}
      onToggle={(value) => { onDepartmentChange(value === "__all__" ? null : value); }}
      options={[
        { value: "__all__", label: t("reports.all_departments", "Tất cả phòng ban") },
        ...departments.map((department) => ({ value: department.id, label: department.name })),
      ]}
      searchable
      searchPlaceholder={t("reports.department_search_placeholder", "Tìm phòng ban...")}
      selected={selectedDepartmentId ? [selectedDepartmentId] : []}
      trigger={
        <FilterTrigger
          aria-haspopup="listbox"
          chevron
          data-testid="department-filter-trigger"
          hasActive={Boolean(selectedDepartmentId)}
          icon={<Building2 aria-hidden="true" size={15} />}
          label={selectedDepartment?.name ?? t("reports.all_departments", "Tất cả phòng ban")}
          size="sm"
          variant="pill-button"
        />
      }
    />
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
  if (!employers?.length) return null;

  const selectedEmployer = employers.find((employer) => employer.id === selectedEmployerId);

  return (
    <FilterPanel
      container="popover"
      mode="single"
      onClear={() => { onEmployerChange?.(null); }}
      onToggle={(value) => { onEmployerChange?.(value === "__all__" ? null : value); }}
      options={[
        { value: "__all__", label: t("reports.all_employers", "Tất cả công ty") },
        ...employers.map((employer) => ({ value: employer.id, label: employer.name })),
      ]}
      searchable
      searchPlaceholder={t("reports.employer_search_placeholder", "Tìm công ty...")}
      selected={selectedEmployerId ? [selectedEmployerId] : []}
      trigger={
        <FilterTrigger
          aria-haspopup="listbox"
          hasActive={Boolean(selectedEmployerId)}
          icon={<Briefcase aria-hidden="true" size={15} />}
          label={selectedEmployer?.name ?? t("reports.all_employers", "Tất cả công ty")}
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
  const activeFilters = [
    filterMode !== "last_month",
    employeeStatus !== "active",
    Boolean(selectedDepartmentId),
    Boolean(selectedEmployerId),
    Boolean(searchQuery.trim()),
  ].filter(Boolean).length;

  const clearFilters = () => {
    onFilterModeChange("last_month");
    onEmployeeStatusChange("active");
    onDepartmentChange(null);
    onEmployerChange?.(null);
    onSearchQueryChange("");
  };

  return (
    <section aria-label={t("common.filters.title", "Bộ lọc báo cáo")} className="space-y-3">
      <ToolbarRow className="items-stretch gap-2 lg:items-center">
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
          <span className="sr-only">{t("reports.date_range", "Khoảng thời gian")}</span>
          <FilterModeButtons
            filterMode={filterMode}
            onFilterModeChange={onFilterModeChange}
            onSpecificMonthChange={onSpecificMonthChange}
            size="sm"
            specificMonth={specificMonth}
            t={t}
          />
          <StatusFilterSelector employeeStatus={employeeStatus} onEmployeeStatusChange={onEmployeeStatusChange} t={t} />
          {!!departments?.length && (
            <DepartmentFilter
              departments={departments}
              onDepartmentChange={onDepartmentChange}
              selectedDepartmentId={selectedDepartmentId}
              t={t}
            />
          )}
          <EmployerFilterSelector
            employers={employers}
            onEmployerChange={onEmployerChange}
            selectedEmployerId={selectedEmployerId}
            t={t}
          />
        </div>

        <div className="w-full shrink-0 sm:max-w-sm lg:w-64 xl:w-72">
          <SearchInput
            aria-label={t("reports.search_label", "Tìm nhân viên")}
            containerClassName="bg-white dark:bg-slate-900"
            data-testid="report-search-shared"
            onChange={(event) => { onSearchQueryChange(event.target.value); }}
            onClear={() => { onSearchQueryChange(""); }}
            onFocus={(event) => { event.currentTarget.select(); }}
            onKeyDown={onSearchKeyDown}
            placeholder={t("reports.search_placeholder", "Tìm theo tên nhân viên...")}
            ref={searchInputRef}
            size="sm"
            value={searchQuery}
          />
        </div>
      </ToolbarRow>

      {activeFilters > 0 && (
        <div className="flex flex-wrap items-center gap-2" data-testid="report-active-filters">
          <span className="text-xs font-semibold text-slate-500 dark:text-slate-400">
            {t("reports.active_filters", "Đang lọc")}
          </span>
          <span className="rounded-full bg-primary-50 px-2.5 py-1 text-xs font-medium text-primary-700 dark:bg-primary-950 dark:text-primary-300">
            {activeFilters}
          </span>
          <Button
            className="min-h-8 gap-1.5 rounded-lg px-2.5 text-xs normal-case tracking-normal"
            onClick={clearFilters}
            type="button"
            variant="ghost"
          >
            <RotateCcw aria-hidden="true" size={13} />
            {t("reports.clear_filters", "Xóa bộ lọc")}
          </Button>
        </div>
      )}

      {filterMode === "custom" && (
        <div className="grid max-w-xl grid-cols-1 gap-3 rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-900 sm:grid-cols-2">
          <div className="space-y-1.5">
            <label className="flex items-center gap-1.5 text-xs font-semibold text-slate-600 dark:text-slate-400" htmlFor="report-start-date">
              <Calendar aria-hidden="true" size={13} /> {t("common.from_date", "Từ ngày")}
            </label>
            <input
              className="min-h-10 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-slate-800 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
              id="report-start-date"
              onChange={(event) => { onDateRangeChange({ ...dateRange, start: event.target.value }); }}
              type="date"
              value={dateRange.start}
            />
          </div>
          <div className="space-y-1.5">
            <label className="flex items-center gap-1.5 text-xs font-semibold text-slate-600 dark:text-slate-400" htmlFor="report-end-date">
              <Calendar aria-hidden="true" size={13} /> {t("common.to_date", "Đến ngày")}
            </label>
            <input
              className="min-h-10 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-slate-800 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
              id="report-end-date"
              onChange={(event) => { onDateRangeChange({ ...dateRange, end: event.target.value }); }}
              type="date"
              value={dateRange.end}
            />
          </div>
        </div>
      )}
    </section>
  );
}
