import { Calendar, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Department } from "@/lib/api/types";

import { EmployeeSearch } from "@/components/common/employee-search";
import { FilterModeButtons } from "@/components/common/filter-mode-buttons";
import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";
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
  wsId: string | null;
}

function CustomRangeFilter({ dateRange, setDateRange }: { dateRange: { start: string, end: string }, setDateRange: React.Dispatch<React.SetStateAction<{ start: string, end: string }>> }) {
  const { t } = useTranslation();
  return (
    <div className="bg-white dark:bg-slate-900 p-6 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-top-2 duration-300 max-w-2xl">
      <div className="space-y-2">
        <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.from_date")}</label>
        <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" onChange={(e) => { setDateRange((prev) => ({ ...prev, start: e.target.value })); }} type="date" value={dateRange.start} />
      </div>
      <div className="space-y-2">
        <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.to_date")}</label>
        <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" onChange={(e) => { setDateRange((prev) => ({ ...prev, end: e.target.value })); }} type="date" value={dateRange.end} />
      </div>
    </div>
  );
}

export function AttendanceLogsFilters(props: AttendanceLogsFiltersProps) {
  const { t } = useTranslation();
  const { dateRange, setDateRange, employeeId, setEmployeeId, filterMode, handleFilterModeChange, handleSpecificMonthChange, specificMonth, wsId, departments } = props;

  const employeeSearch = (
    <div className="flex w-full items-center gap-3">
      <div className="min-w-0 flex-1">
        <EmployeeSearch clearOnEscape data-testid="employee-search" departments={departments} keyboardShortcut={["/", "Enter"]} onChange={(value) => { setEmployeeId((Array.isArray(value) ? value[0] : value) || "all"); }} placeholder={employeeId === "all" ? t("common.all_employees") : undefined} value={employeeId === "all" ? "" : employeeId} workspaceId={wsId ?? ""} />
      </div>
      {employeeId !== "all" && (<Button className="p-2.5 bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl transition-all" onClick={() => { setEmployeeId("all"); }} type="button" variant="ghost"><Users size={18} /></Button>)}
    </div>
  );

  return (
    <ResponsiveToolbar
      className="rounded-[2rem] border border-slate-200 bg-white p-2 shadow-sm dark:border-slate-700 dark:bg-slate-900"
      filterCount={(filterMode !== "this_month" ? 1 : 0) + (employeeId !== "all" ? 1 : 0)}
      searchSlot={employeeSearch}
    >
      <div className="space-y-4">
        <FilterModeButtons filterMode={filterMode} onFilterModeChange={handleFilterModeChange} onSpecificMonthChange={handleSpecificMonthChange} specificMonth={specificMonth} t={t} />
        <div className="lg:hidden">{employeeSearch}</div>
        {filterMode === "custom" ? <CustomRangeFilter dateRange={dateRange} setDateRange={setDateRange} /> : null}
      </div>
    </ResponsiveToolbar>
  );
}
