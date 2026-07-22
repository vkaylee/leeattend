import { Briefcase, ChevronRight, Clock3, Palmtree } from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import NavigableList from "@/components/common/navigable-list";
import ReportFilters from "@/components/features/reports/report-filters";
import Button from "@/components/ui/button";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";
import { AttendanceSummary, AttendanceReportSort, Department, Employer } from "@/lib/api/types";
import { formatHours } from "@/lib/utils/attendance";

import { FilterMode } from "../hooks/use-reports-page";
import { ReportsStats } from "./reports-stats";
import { getReportColumns } from "./reports-table/columns";

function ReportMobileCard({ row, onSelect, t }: {
  onSelect: () => void;
  row: AttendanceSummary;
  t: (key: string) => string;
}) {
  const totalHours = row.computed_std_hours + row.computed_ot_hours;

  return (
    <Button
      className="h-auto w-full justify-start rounded-2xl border border-slate-100 bg-white p-5 text-left normal-case tracking-normal shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900"
      onClick={onSelect}
      type="button"
      variant="ghost"
    >
      <div className="flex items-start gap-3">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 font-bold uppercase text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400">
          {row.employee_name.split(" ").map((part) => part[0]).join("").slice(0, 2)}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">{row.employee_name}</div>
          <div className="mt-1 font-mono text-[10px] font-bold uppercase tracking-wider text-primary-600 dark:text-primary-400">{row.employee_id_number ?? "—"}</div>
          {!!row.employer_name && (
            <div className="mt-1 flex items-center gap-1 truncate text-[10px] font-medium text-slate-500 dark:text-slate-400">
              <Briefcase size={11} /> {row.employer_name}
            </div>
          )}
        </div>
        <ChevronRight className="mt-1 h-4 w-4 shrink-0 text-slate-300 dark:text-slate-600" />
      </div>

      <div className="mt-4 grid grid-cols-3 gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="rounded-xl bg-slate-50 p-3 dark:bg-slate-800/60">
          <div className="flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider text-slate-400"><Clock3 size={10} />{t("reports.table.total_hours")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">{formatHours(totalHours)}h</div>
        </div>
        <div className="rounded-xl bg-orange-50 p-3 dark:bg-orange-950/30">
          <div className="text-[9px] font-bold uppercase tracking-wider text-orange-500">{t("reports.table.ot_hours")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-orange-600 dark:text-orange-400">{formatHours(row.computed_ot_hours)}h</div>
        </div>
        <div className="rounded-xl bg-primary-50 p-3 dark:bg-primary-950/30">
          <div className="flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider text-primary-500"><Palmtree size={10} />{t("reports.table.leave_days")}</div>
          <div className="mt-1 font-mono text-sm font-bold text-primary-600 dark:text-primary-400">{row.total_leave_days}</div>
        </div>
      </div>
    </Button>
  );
}

interface ReportsTableProps {
  dateRange: { end: string; start: string };
  departments: Department[] | undefined;
  employeeStatus: string;
  employers: Employer[] | undefined;
  fetchNextPage: () => void;
  filteredData: AttendanceSummary[];
  filterMode: FilterMode;
  handleDateRangeChange: (range: { end: string; start: string }) => void;
  handleDepartmentChange: (id: string | null) => void;
  handleEmployeeClick: (employeeId: string, employeeName: string, employerId?: string) => void;
  handleEmployerChange: (id: string | null) => void;
  handleFilterModeChange: (mode: FilterMode) => void;
  handleSort: (field: "leave_days" | "name" | "ot_hours" | "total_hours" | "work_hours") => void;
  handleSpecificMonthChange: (m: number, y: number) => void;
  hasNextPage: boolean | undefined;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  keyDownRef: React.RefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>;
  onEmployeeStatusChange: (status: string) => void;
  quickStats: { otHours: string; stdHours: string; totalEmployees: number };
  searchInputRef: React.RefObject<HTMLInputElement | null>;
  searchQuery: string;
  selectedDepartmentId: string | null;
  selectedEmployerId: string | null;
  setSearchQuery: (q: string) => void;
  sortBy: AttendanceReportSort | undefined;
  specificMonth: { month: number; year: number };
}

export function ReportsTable(props: ReportsTableProps) {
  const { t } = useTranslation();
  const {
    dateRange,
    departments,
    employers,
    selectedEmployerId,
    handleEmployerChange,
    fetchNextPage,
    filteredData,
    filterMode,
    handleDateRangeChange,
    handleDepartmentChange,
    handleEmployeeClick,
    handleFilterModeChange,
    handleSort,
    handleSpecificMonthChange,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
    keyDownRef,
    quickStats,
    searchInputRef,
    searchQuery,
    selectedDepartmentId,
    setSearchQuery,
    sortBy,
    specificMonth,
    employeeStatus,
    onEmployeeStatusChange,
  } = props;

  const reportColumns = useMemo(() => getReportColumns({
    handleSort,
    sortBy,
    t,
  }), [t, sortBy, handleSort]);

  return (
    <div className="flex flex-col gap-6">
      <ResponsiveToolbar
        filterCount={[
          filterMode !== "this_month",
          employeeStatus !== "active",
          selectedDepartmentId !== null,
          selectedEmployerId !== null,
        ].filter(Boolean).length}
      >
        <ReportFilters
          dateRange={dateRange}
          departments={departments}
          employeeStatus={employeeStatus}
          employers={employers}
          filterMode={filterMode}
          onDateRangeChange={handleDateRangeChange}
          onDepartmentChange={handleDepartmentChange}
          onEmployeeStatusChange={onEmployeeStatusChange}
          onEmployerChange={handleEmployerChange}
          onFilterModeChange={handleFilterModeChange}
          onSearchKeyDown={(e) => keyDownRef.current?.(e)}
          onSearchQueryChange={setSearchQuery}
          onSpecificMonthChange={handleSpecificMonthChange}
          searchInputRef={searchInputRef}
          searchQuery={searchQuery}
          selectedDepartmentId={selectedDepartmentId}
          selectedEmployerId={selectedEmployerId}
          specificMonth={specificMonth}
          t={t}
        />
      </ResponsiveToolbar>
      <NavigableList<AttendanceSummary>
        className="animate-in slide-in-from-bottom-8 duration-700"
        columns={reportColumns}
        emptyMessage={t("reports.messages.no_data")}
        fetchNextPage={fetchNextPage}
        hasNextPage={hasNextPage}
        headerSlot={<div className="flex items-center gap-3 shrink-0"><ReportsStats quickStats={quickStats} /></div>}
        isFetchingNextPage={isFetchingNextPage}
        isLoading={isLoading}
        itemKey={(row) => `${row.employee_id}:${row.employer_id ?? "none"}`}
        items={filteredData}
        maxHeightClass=""
        onKeyDownRef={keyDownRef}
        onSearchChange={setSearchQuery}
        onSelect={(row) => { handleEmployeeClick(row.employee_id, row.employee_name, row.employer_id); }}
        renderMobileCard={(row) => (
          <ReportMobileCard
            onSelect={() => { handleEmployeeClick(row.employee_id, row.employee_name, row.employer_id); }}
            row={row}
            t={t}
          />
        )}
        resetDeps={[searchQuery, filterMode, specificMonth, dateRange]}
        rowIdPrefix="summary-row"
        searchInputRef={searchInputRef}
        searchPlaceholder={t("reports.search_placeholder")}
        searchTestId="report-search"
        searchValue={searchQuery}
        showChevron
      />
    </div>
  );
}
