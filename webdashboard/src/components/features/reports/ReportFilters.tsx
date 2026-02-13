import { Calendar, CalendarRange, List, TableProperties } from "lucide-react";
import { cn } from "@/lib/utils";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface ReportFiltersProps {
  filterMode: "this_month" | "last_month" | "custom" | "specific_month";
  reportType: "summary" | "daily";
  dateRange: { start: string; end: string };
  specificMonth: { month: number; year: number };
  selectedEmployeeIds: string[];
  workspaceId: string;
  t: (key: string) => string;
  onFilterModeChange: (mode: "this_month" | "last_month" | "custom" | "specific_month") => void;
  onReportTypeChange: (type: "summary" | "daily") => void;
  onDateRangeChange: (range: { start: string; end: string }) => void;
  onSpecificMonthChange: (month: number, year: number) => void;
  onEmployeeIdsChange: (ids: string[]) => void;
}

export default function ReportFilters({
  filterMode,
  reportType,
  dateRange,
  specificMonth,
  selectedEmployeeIds,
  workspaceId,
  t,
  onFilterModeChange,
  onReportTypeChange,
  onDateRangeChange,
  onSpecificMonthChange,
  onEmployeeIdsChange,
}: ReportFiltersProps) {
  return (
    <>
      <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-8">
        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
          <button
            onClick={() => onReportTypeChange("summary")}
            className={cn(
              "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
              reportType === 'summary'
                ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                : "text-slate-500 hover:text-slate-700"
            )}
          >
            <List size={14} /> {t("reports.tabs.summary")}
          </button>

          <button
            onClick={() => onReportTypeChange("daily")}
            className={cn(
              "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
              reportType === 'daily'
                ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                : "text-slate-500 hover:text-slate-700"
            )}
          >
            <TableProperties size={14} /> {t("reports.tabs.daily")}
          </button>
        </div>

        <div className="flex flex-col md:flex-row md:items-center gap-6 bg-white p-3 rounded-[2.5rem] border border-slate-100 shadow-xl shadow-slate-900/5 pr-6">
          <div className="flex items-center bg-slate-100/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200/50">
            {["this_month", "last_month"].map(mode => (
              <button
                key={mode}
                onClick={() => onFilterModeChange(mode as any)}
                className={cn(
                  "px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
                  filterMode === mode
                    ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                    : "text-slate-500 hover:text-slate-700"
                )}
              >
                {t(`common.filters.${mode}`).toUpperCase()}
              </button>
            ))}

            <div className="h-6 w-px bg-slate-200 mx-2 opacity-50" />

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  className={cn(
                    "flex items-center gap-2 px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
                    filterMode === "specific_month"
                      ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                      : "text-slate-500 hover:text-slate-700"
                  )}
                >
                  <CalendarRange size={14} />
                  {filterMode === "specific_month"
                    ? `${specificMonth.month}/${specificMonth.year}`
                    : t("common.filters.specific_month").toUpperCase()}
                </button>
              </DropdownMenuTrigger>

              <DropdownMenuContent
                className="w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border-slate-100 bg-white/95 backdrop-blur-xl"
                align="end"
                sideOffset={12}
              >
                <MonthYearPicker
                  month={specificMonth.month}
                  year={specificMonth.year}
                  onChange={onSpecificMonthChange}
                />
              </DropdownMenuContent>
            </DropdownMenu>

            <button
              onClick={() => onFilterModeChange("custom")}
              className={cn(
                "px-5 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
                filterMode === "custom"
                  ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                  : "text-slate-500 hover:text-slate-700"
              )}
            >
              {t("common.filters.custom").toUpperCase()}
            </button>
          </div>

          <div className="h-10 w-px bg-slate-100 hidden md:block opacity-50" />

          <div className="w-full md:w-80">
            <EmployeeSearch
              workspaceId={workspaceId}
              value={selectedEmployeeIds}
              onChange={onEmployeeIdsChange}
              multiple={true}
              autoFocus={true}
              placeholder={t("reports.search_placeholder")}
            />
          </div>
        </div>
      </div>

      {filterMode === "custom" && (
        <div className="bg-white p-8 rounded-[2.5rem] border border-slate-100 shadow-xl shadow-primary-900/5 grid grid-cols-1 md:grid-cols-2 gap-10 animate-in slide-in-from-top-4 duration-500 max-w-2xl">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 flex items-center gap-2">
              <Calendar size={12} /> {t("common.from_date")}
            </label>
            <input
              type="date"
              value={dateRange.start}
              onChange={(e) => onDateRangeChange({ ...dateRange, start: e.target.value })}
              className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white transition-all font-bold text-sm shadow-inner"
            />
          </div>
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 flex items-center gap-2">
              <Calendar size={12} /> {t("common.to_date")}
            </label>
            <input
              type="date"
              value={dateRange.end}
              onChange={(e) => onDateRangeChange({ ...dateRange, end: e.target.value })}
              className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white transition-all font-bold text-sm shadow-inner"
            />
          </div>
        </div>
      )}
    </>
  );
}
