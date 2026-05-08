import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import NavigableList from "@/components/common/navigable-list";
import ReportFilters from "@/components/features/reports/report-filters";
import { AttendanceSummary, AttendanceReportSort, Department } from "@/lib/api/types";

import { FilterMode } from "../hooks/use-reports-page";
import { ReportsStats } from "./reports-stats";
import { getReportColumns } from "./reports-table/columns";

interface ReportsTableProps {
  dateRange: { end: string; start: string };
  departments: Department[] | undefined;
  fetchNextPage: () => void;
  filteredData: AttendanceSummary[];
  filterMode: FilterMode;
  handleDateRangeChange: (range: { end: string; start: string }) => void;
  handleDepartmentChange: (id: string | null) => void;
  handleEmployeeClick: (employeeId: string, employeeName: string) => void;
  handleFilterModeChange: (mode: FilterMode) => void;
  handleSort: (field: "leave_days" | "name" | "ot_hours" | "total_hours" | "work_hours") => void;
  handleSpecificMonthChange: (m: number, y: number) => void;
  hasNextPage: boolean | undefined;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  keyDownRef: React.RefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>;
  quickStats: { otHours: string; stdHours: string; totalEmployees: number };
  searchInputRef: React.RefObject<HTMLInputElement | null>;
  searchQuery: string;
  selectedDepartmentId: string | null;
  setSearchQuery: (q: string) => void;
  sortBy: AttendanceReportSort | undefined;
  specificMonth: { month: number; year: number };
  workUnitMinutes: number;
}

export function ReportsTable(props: ReportsTableProps) {
  const { t } = useTranslation();
  const {
    dateRange,
    departments,
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
    workUnitMinutes,
  } = props;

  const reportColumns = useMemo(() => getReportColumns({
    handleSort,
    sortBy,
    t,
    workUnitMinutes,
  }), [t, workUnitMinutes, sortBy, handleSort]);

  return (
    <div className="flex flex-col gap-6">
      <ReportFilters
        dateRange={dateRange}
        departments={departments}
        filterMode={filterMode}
        onDateRangeChange={handleDateRangeChange}
        onDepartmentChange={handleDepartmentChange}
        onFilterModeChange={handleFilterModeChange}
        onSearchKeyDown={(e) => keyDownRef.current?.(e)}
        onSearchQueryChange={setSearchQuery}
        onSpecificMonthChange={handleSpecificMonthChange}
        searchInputRef={searchInputRef}
        searchQuery={searchQuery}
        selectedDepartmentId={selectedDepartmentId}
        specificMonth={specificMonth}
        statsSlot={<ReportsStats quickStats={quickStats} />}
        t={t}
      />
      <NavigableList<AttendanceSummary>
        className="animate-in slide-in-from-bottom-8 duration-700"
        columns={reportColumns}
        emptyMessage={t("reports.messages.no_data")}
        fetchNextPage={fetchNextPage}
        hasNextPage={hasNextPage}
        isFetchingNextPage={isFetchingNextPage}
        isLoading={isLoading}
        itemKey={(row) => row.employee_id}
        items={filteredData}
        maxHeightClass=""
        onKeyDownRef={keyDownRef}
        onSearchChange={setSearchQuery}
        onSelect={(row) => { handleEmployeeClick(row.employee_id, row.employee_name); }}
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
