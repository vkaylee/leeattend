import { Calendar, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Department } from "@/lib/api/types";

import { EmployeeSearch } from "@/components/common/employee-search";
import { FilterModeButtons } from "@/components/common/filter-mode-buttons";
import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";
import { ToolbarRow } from "@/components/ui/toolbar-row";
import { cn } from "@/lib/utils";

import type { FilterMode } from "../hooks/use-attendance-logs-page";

interface AttendanceLogsFiltersProps {
  dateRange: { start: string; end: string };
  departments?: Department[];
  employeeId: string;
  filterMode: FilterMode;
  handleFilterModeChange: (mode: FilterMode) => void;
  handleSpecificMonthChange: (m: number, y: number) => void;
  setDateRange: React.Dispatch<React.SetStateAction<{ start: string; end: string }>>;
  setEmployeeId: (id: string) => void;
  specificMonth: { month: number; year: number };
}

function CustomRangeFilter({ dateRange, setDateRange }: { dateRange: { start: string, end: string }, setDateRange: React.Dispatch<React.SetStateAction<{ start: string, end: string }>> }) {
  const { t } = useTranslation();
  return (
    <div className="grid max-w-2xl grid-cols-1 gap-3 rounded-2xl border border-slate-200 bg-white p-3 shadow-sm animate-in slide-in-from-top-2 duration-300 dark:border-slate-700 dark:bg-slate-900 md:grid-cols-2">
      <div className="space-y-1.5">
        <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.from_date")}</label>
        <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 h-[var(--ui-height-sm)] rounded-xl px-4 outline-none focus:ring-2 focus:ring-primary-500/20 transition-all font-medium text-sm" onChange={(e) => { setDateRange((prev) => ({ ...prev, start: e.target.value })); }} type="date" value={dateRange.start} />
      </div>
      <div className="space-y-1.5">
        <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.to_date")}</label>
        <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 h-[var(--ui-height-sm)] rounded-xl px-4 outline-none focus:ring-2 focus:ring-primary-500/20 transition-all font-medium text-sm" onChange={(e) => { setDateRange((prev) => ({ ...prev, end: e.target.value })); }} type="date" value={dateRange.end} />
      </div>
    </div>
  );
}

export function AttendanceLogsFilters(props: AttendanceLogsFiltersProps) {
  const { t } = useTranslation();
  const { dateRange, setDateRange, employeeId, setEmployeeId, filterMode, handleFilterModeChange, handleSpecificMonthChange, specificMonth, departments } = props;

  const employeeSearch = (
    <div className="flex h-[var(--ui-height-sm)] min-w-0 w-full flex-1 items-center gap-3 self-center">
      <div className="min-w-0 flex-1 self-center">
        <EmployeeSearch clearOnEscape data-testid="employee-search" departments={departments} keyboardShortcut={["/", "Enter"]} onChange={(value) => { setEmployeeId((Array.isArray(value) ? value[0] : value) || "all"); }} placeholder={employeeId === "all" ? t("common.all_employees") : undefined} size="sm" value={employeeId === "all" ? "" : employeeId}  />
      </div>
      {employeeId !== "all" && (<Button aria-label={t("common.clear")} className="shrink-0" icon={<Users size={18} />} onClick={() => { setEmployeeId("all"); }} size="sm" type="button" variant="ghost" />)}
    </div>
  );

  return (
    <ResponsiveToolbar
      className="items-center rounded-2xl border border-slate-200 bg-white p-1 shadow-sm dark:border-slate-700 dark:bg-slate-900"
      desktopContent={
        <ToolbarRow className="flex-nowrap">
          {employeeSearch}
          <FilterModeButtons filterMode={filterMode} onFilterModeChange={handleFilterModeChange} onSpecificMonthChange={handleSpecificMonthChange} size="sm" specificMonth={specificMonth} t={t} />
        </ToolbarRow>
      }
      filterCount={(filterMode !== "this_month" ? 1 : 0) + (employeeId !== "all" ? 1 : 0)}
    >
      <div className="flex items-center gap-3">
        <FilterModeButtons filterMode={filterMode} onFilterModeChange={handleFilterModeChange} onSpecificMonthChange={handleSpecificMonthChange} size="sm" specificMonth={specificMonth} t={t} />
        <div className="lg:hidden">{employeeSearch}</div>
      </div>
      {filterMode === "custom" ? <CustomRangeFilter dateRange={dateRange} setDateRange={setDateRange} /> : null}
    </ResponsiveToolbar>
  );
}
