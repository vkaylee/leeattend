import { Calendar, Building2, Briefcase } from "lucide-react";
import React from "react";

import type { Department, Employer } from "@/lib/api/types";

import { FilterModeButtons } from "@/components/common/filter-mode-buttons";
import Button from "@/components/ui/button";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
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
  t: (key: string) => string;
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
    <div className="flex flex-wrap items-center gap-1 bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
      {options.map(status => (
        <Button
          className={cn(
            "px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all h-auto",
            employeeStatus === status
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )}
          key={status}
          onClick={() => { onEmployeeStatusChange(status); }}
          variant="ghost"
        >
          {t(`reports.filters.status.${status}`).toUpperCase()}
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
    <div className="flex flex-wrap items-center gap-2 pb-1" data-testid="department-chips">
      <Building2 className="text-slate-400 dark:text-slate-500 shrink-0" size={14} />
      <Button
        className={cn(
          "px-4 py-2 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all border h-auto",
          selectedDepartmentId === null
            ? "bg-primary-600 dark:bg-primary-500 text-white border-primary-600 dark:border-primary-500 shadow-lg shadow-primary-500/20 hover:bg-primary-700 dark:hover:bg-primary-600 hover:text-white"
            : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
        )}
        onClick={() => { onDepartmentChange(null); }}
        variant="ghost"
      >
        {t("reports.all_departments")}
      </Button>
      {departments.map((dept) => (
        <Button
          className={cn(
            "px-4 py-2 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all border h-auto",
            selectedDepartmentId === dept.id
              ? "bg-primary-600 dark:bg-primary-500 text-white border-primary-600 dark:border-primary-500 shadow-lg shadow-primary-500/20 hover:bg-primary-700 dark:hover:bg-primary-600 hover:text-white"
              : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
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
      options={[
        { value: "__all__", label: t("reports.all_employers") },
        ...employers.map(e => ({ value: e.id, label: e.name })),
      ]}
      selected={selectedEmployerId ? [selectedEmployerId] : []}
      onToggle={(val) => onEmployerChange?.(val === "__all__" ? null : val)}
      onClear={() => onEmployerChange?.(null)}
      trigger={
        <FilterTrigger
          variant="pill-button"
          label={selectedEmployer ? selectedEmployer.name : t("reports.all_employers")}
          hasActive={!!selectedEmployerId}
          icon={<Briefcase size={14} />}
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
}: ReportFiltersProps) {
  return (
    <>
      <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-8">
        <div className="flex flex-col md:flex-row md:items-center gap-6 bg-white dark:bg-slate-900 p-3 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
          <FilterModeButtons className="rounded-[1.5rem] p-1.5" filterMode={filterMode} onFilterModeChange={onFilterModeChange} onSpecificMonthChange={onSpecificMonthChange} specificMonth={specificMonth} t={t} uppercase />
          <div className="hidden md:block w-px h-8 bg-slate-200 dark:bg-slate-800 self-center" />
          <StatusFilterSelector employeeStatus={employeeStatus} onEmployeeStatusChange={onEmployeeStatusChange} t={t} />
          {!!employers && employers.length > 0 && (
            <>
              <div className="hidden md:block w-px h-8 bg-slate-200 dark:bg-slate-800 self-center" />
              <EmployerFilterSelector employers={employers} onEmployerChange={onEmployerChange} selectedEmployerId={selectedEmployerId} t={t} />
            </>
          )}
        </div>
      </div>

      {departments && departments.length > 0 ? (
        <DepartmentChips departments={departments} onDepartmentChange={onDepartmentChange} selectedDepartmentId={selectedDepartmentId} t={t} />
      ) : null}

      {filterMode === "custom" ? (
        <div className="bg-white dark:bg-slate-900 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-primary-900/5 grid grid-cols-1 md:grid-cols-2 gap-10 animate-in slide-in-from-top-4 duration-500 max-w-2xl">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
              <Calendar size={12} /> {t("common.from_date")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-sm shadow-inner"
              onChange={(e) => { onDateRangeChange({ ...dateRange, start: e.target.value }); }}
              type="date"
              value={dateRange.start}
            />
          </div>
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
              <Calendar size={12} /> {t("common.to_date")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-sm shadow-inner"
              onChange={(e) => { onDateRangeChange({ ...dateRange, end: e.target.value }); }}
              type="date"
  // eslint-disable-next-line max-lines
              value={dateRange.end}
            />
          </div>
        </div>
      ) : null}
    </>
  );
}
