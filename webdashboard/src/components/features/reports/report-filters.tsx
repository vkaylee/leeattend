import { Calendar, CalendarRange, Building2 } from "lucide-react";
import React from "react";

import type { Department } from "@/lib/api/types";

import { MonthYearPicker } from "@/components/common/month-year-picker";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

interface ReportFiltersProps {
  dateRange: { start: string; end: string };
  departments?: Department[];
  filterMode: "this_month" | "last_month" | "custom" | "specific_month";
  onDateRangeChange: (range: { start: string; end: string }) => void;
  onDepartmentChange: (departmentId: string | null) => void;
  onFilterModeChange: (mode: "this_month" | "last_month" | "custom" | "specific_month") => void;
  onSearchKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  onSearchQueryChange: (query: string) => void;
  onSpecificMonthChange: (month: number, year: number) => void;
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  searchQuery: string;
  selectedDepartmentId: string | null;
  specificMonth: { month: number; year: number };
  statsSlot?: React.ReactNode;
  t: (key: string) => string;
}

function FilterModeSelector({
  filterMode,
  onFilterModeChange,
  onSpecificMonthChange,
  specificMonth,
  t,
}: {
  filterMode: ReportFiltersProps["filterMode"];
  onFilterModeChange: ReportFiltersProps["onFilterModeChange"];
  onSpecificMonthChange: ReportFiltersProps["onSpecificMonthChange"];
  specificMonth: ReportFiltersProps["specificMonth"];
  t: ReportFiltersProps["t"];
}) {
  return (
    <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
      {(["this_month", "last_month"] as const).map(mode => (
        <button
          className={cn(
            "px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
            filterMode === mode
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )}
          key={mode}
          onClick={() => { onFilterModeChange(mode); }}
        >
          {t(`common.filters.${mode}`).toUpperCase()}
        </button>
      ))}
      <div className="h-6 w-px bg-slate-200 dark:bg-slate-600 mx-2 opacity-50" />
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button
            className={cn(
              "flex items-center gap-2 px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
              filterMode === "specific_month"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
          >
            <CalendarRange size={14} />
            {filterMode === "specific_month"
              ? `${specificMonth.month}/${specificMonth.year}`
              : t("common.filters.specific_month").toUpperCase()}
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="end"
          className="w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/95 backdrop-blur-xl"
          sideOffset={12}
        >
          <MonthYearPicker month={specificMonth.month} onChange={onSpecificMonthChange} year={specificMonth.year} />
        </DropdownMenuContent>
      </DropdownMenu>
      <button
        className={cn(
          "px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
          filterMode === "custom"
            ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
        )}
        onClick={() => { onFilterModeChange("custom"); }}
      >
        {t("common.filters.custom").toUpperCase()}
      </button>
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
      <button
        className={cn(
          "px-4 py-2 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all border",
          selectedDepartmentId === null
            ? "bg-primary-600 dark:bg-primary-500 text-white border-primary-600 dark:border-primary-500 shadow-lg shadow-primary-500/20"
            : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
        )}
        onClick={() => { onDepartmentChange(null); }}
      >
        {t("reports.all_departments")}
      </button>
      {departments.map((dept) => (
        <button
          className={cn(
            "px-4 py-2 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all border",
            selectedDepartmentId === dept.id
              ? "bg-primary-600 dark:bg-primary-500 text-white border-primary-600 dark:border-primary-500 shadow-lg shadow-primary-500/20"
              : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
          )}
          key={dept.id}
          onClick={() => { onDepartmentChange(dept.id); }}
        >
          {dept.name}
        </button>
      ))}
    </div>
  );
}

export default function ReportFilters({
  dateRange,
  departments,
  filterMode,
  onDateRangeChange,
  onDepartmentChange,
  onFilterModeChange,
  onSpecificMonthChange,
  selectedDepartmentId,
  specificMonth,
  statsSlot,
  t,
}: ReportFiltersProps) {
  return (
    <>
      <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-8">
        <div className="flex flex-col md:flex-row md:items-center gap-6 bg-white dark:bg-slate-900 p-3 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
          <FilterModeSelector filterMode={filterMode} onFilterModeChange={onFilterModeChange} onSpecificMonthChange={onSpecificMonthChange} specificMonth={specificMonth} t={t} />
        </div>
        {statsSlot ? <div className="flex items-center gap-3 shrink-0">{statsSlot}</div> : null}
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
              value={dateRange.end}
            />
          </div>
        </div>
      ) : null}
    </>
  );
}
