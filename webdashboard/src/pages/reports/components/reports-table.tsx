import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import NavigableList from "@/components/common/navigable-list";
import ReportFilters from "@/components/features/reports/report-filters";
import { AttendanceSummary, AttendanceReportSort, Department, Employer } from "@/lib/api/types";

import { FilterMode } from "../hooks/use-reports-page";
import { ReportsStats } from "./reports-stats";
import { getReportColumns } from "./reports-table/columns";

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
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        onSelect={(row) => { handleEmployeeClick(row.employee_id ?? "", row.employee_name, row.employer_id); }}
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
