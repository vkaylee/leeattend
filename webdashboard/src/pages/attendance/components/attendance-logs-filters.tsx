import { Calendar, CalendarRange, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { cn } from "@/lib/utils";

import type { FilterMode } from "../hooks/use-attendance-logs-page";

interface AttendanceLogsFiltersProps {
  dateRange: { start: string; end: string };
  employeeId: string;
  filterMode: FilterMode;
  handleFilterModeChange: (mode: FilterMode) => void;
  handleSpecificMonthChange: (m: number, y: number) => void;
  setDateRange: React.Dispatch<React.SetStateAction<{ start: string; end: string }>>;
  setEmployeeId: (id: string) => void;
  specificMonth: { month: number; year: number };
  wsId: string | null;
}

function DateFilterButtons({ filterMode, handleFilterModeChange, specificMonth, handleSpecificMonthChange }: { filterMode: FilterMode, handleFilterModeChange: (mode: FilterMode) => void, specificMonth: { month: number, year: number }, handleSpecificMonthChange: (m: number, y: number) => void }) {
  const { t } = useTranslation();
  const now = new Date();
  const curMonth = now.getMonth() + 1; const curYear = now.getFullYear();
  const prev = new Date(now); prev.setMonth(prev.getMonth() - 1);
  const prevMonth = prev.getMonth() + 1; const prevYear = prev.getFullYear();
  const modeLabel = (mode: string) => mode === "this_month" ? `Tháng ${curMonth}/${curYear}` : `Tháng ${prevMonth}/${prevYear}`;
  return (
    <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-2xl p-1 shadow-inner border border-slate-200 dark:border-slate-700/50">
      {(["this_month", "last_month"] as const).map((mode) => (
        <Button className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === mode ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")} key={mode} onClick={() => { handleFilterModeChange(mode); }} type="button" variant="ghost">{modeLabel(mode)}</Button>
      ))}
      <div className="h-6 w-px bg-slate-200 dark:bg-slate-600 mx-1" />
      <div className="relative group">
        <Button className={cn("flex items-center gap-2 px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "specific_month" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")} type="button" variant="ghost"><CalendarRange size={14} /> {filterMode === "specific_month" ? `T${specificMonth.month}/${specificMonth.year}` : t("common.filters.specific_month")}</Button>
        <div className="absolute left-0 top-full mt-2 z-[100] hidden group-hover:block animate-in zoom-in-95 duration-200 origin-top-left"><MonthYearPicker month={specificMonth.month} onChange={handleSpecificMonthChange} year={specificMonth.year} /></div>
      </div>
      <Button className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "custom" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")} onClick={() => { handleFilterModeChange("custom"); }} type="button" variant="ghost">{t("common.filters.custom")}</Button>
    </div>
  );
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
  const { dateRange, setDateRange, employeeId, setEmployeeId, filterMode, handleFilterModeChange, handleSpecificMonthChange, specificMonth, wsId } = props;

  return (
    <>
      <div className="bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm p-2 flex flex-col xl:flex-row xl:items-center justify-between gap-4">
        <DateFilterButtons filterMode={filterMode} handleFilterModeChange={handleFilterModeChange} handleSpecificMonthChange={handleSpecificMonthChange} specificMonth={specificMonth} />
        <div className="flex-1 max-w-md flex items-center gap-3 px-2">
          <div className="flex-1"><EmployeeSearch clearOnEscape data-testid="employee-search" keyboardShortcut={["/", "Enter"]} onChange={(value) => { setEmployeeId((Array.isArray(value) ? value[0] : value) || "all"); }} placeholder={employeeId === "all" ? t("common.all_employees") : undefined} value={employeeId === "all" ? "" : employeeId} workspaceId={wsId ?? ""} /></div>
          {employeeId !== "all" && (<Button className="p-2.5 bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl transition-all" onClick={() => { setEmployeeId("all"); }} type="button" variant="ghost"><Users size={18} /></Button>)}
        </div>
      </div>
      {filterMode === "custom" && <CustomRangeFilter dateRange={dateRange} setDateRange={setDateRange} />}
    </>
  );
}
