import { lazy, Suspense, useCallback, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";

import {
  TimesheetMatrixGrid,
  type TimesheetMatrixGridHandle,
} from "@/components/features/attendance/timesheet-matrix-grid";
const AttendanceClosurePanel = lazy(() => import("@/components/features/reports/attendance-closure-panel"));
const EmployeeDailyDrawer = lazy(() => import("@/components/features/reports/employee-daily-drawer"));
const ReportExportDialog = lazy(() => import("@/components/features/reports/report-export-dialog"));
import ReportFilters from "@/components/features/reports/report-filters";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";

import { ReportsHeader } from "./reports/components/reports-header";
import { ReportsTable } from "./reports/components/reports-table";
import { useMonthlyTimesheetData } from "./reports/hooks/use-monthly-timesheet-data";
import { filterMatrixEmployees, getActiveMonthYear } from "./reports/report-page-utils";
import { useReportsPage } from "./reports/hooks/use-reports-page";

export default function ReportsPage() {
  const { t } = useTranslation();
  const {
    allEmployees,
    dateRange,
    departments,
    employeeStatus,
    employers,
    fetchNextPage,
    filteredData,
    filterMode,
    handleDateRangeChange,
    handleDepartmentChange,
    handleDrawerClose,
    handleEmployeeClick,
    handleEmployerChange,
    handleFilterModeChange,
    handleSort,
    handleSpecificMonthChange,
    hasNextPage,
    isExportDialogOpen,
    isFetchingNextPage,
    isLoading,
    keyDownRef,
    quickStats,
    searchInputRef,
    searchQuery,
    selectedDepartmentId,
    selectedEmployee,
    selectedEmployerId,
    setEmployeeStatus,
    setIsExportDialogOpen,
    setSearchQuery,
    setViewMode,
    sortBy,
    specificMonth,
    viewMode,
  } = useReportsPage();

  const matrixGridRef = useRef<TimesheetMatrixGridHandle>(null);

  const activeMonthYear = useMemo(
    () => getActiveMonthYear(filterMode, dateRange.start, specificMonth),
    [dateRange.start, filterMode, specificMonth],
  );

  const { attendanceMap, isLoading: isLoadingAttendance } = useMonthlyTimesheetData({
    month: activeMonthYear.month,
    year: activeMonthYear.year,
    enabled: viewMode === "matrix",
  });

  const matrixFilteredEmployees = useMemo(
    () => filterMatrixEmployees(allEmployees, {
      employeeStatus,
      searchQuery,
      selectedDepartmentId,
      selectedEmployerId,
    }),
    [allEmployees, employeeStatus, searchQuery, selectedDepartmentId, selectedEmployerId],
  );

  // Context-aware export trigger: CSV for Matrix, Dialog for Summary
  const handleExport = useCallback(() => {
    if (viewMode === "matrix") {
      matrixGridRef.current?.exportCSV();
    } else {
      setIsExportDialogOpen(true);
    }
  }, [viewMode, setIsExportDialogOpen]);

  const clickedEmployeeSummaryRows = selectedEmployee?.id
    ? filteredData.filter((row) => row.employee_id === selectedEmployee.id)
    : [];

  const clickedEmployers = clickedEmployeeSummaryRows.map((row) => ({
    employeeIdNumber: row.employee_id_number,
    employerId: row.employer_id,
    employerName: row.employer_name ?? t("common.employer"),
  }));



  return (
    <div className="space-y-6 pb-20 animate-in fade-in duration-700" data-testid="reports-page">
      <ReportsHeader
        dateRange={dateRange}
        onExportClick={handleExport}
        onViewModeChange={setViewMode}
        viewMode={viewMode}
      />

      {/* Shared Responsive Toolbar & Filters for both Matrix and Summary views */}
      <ResponsiveToolbar
        filterCount={
          [
            filterMode !== "last_month",
            employeeStatus !== "active",
            selectedDepartmentId !== null,
            selectedEmployerId !== null,
            Boolean(searchQuery.trim()),
          ].filter(Boolean).length
        }
      >
        <ReportFilters
          dateRange={dateRange}
          departments={departments}
          employeeStatus={employeeStatus}
          employers={employers}
          filterMode={filterMode}
          onDateRangeChange={handleDateRangeChange}
          onDepartmentChange={handleDepartmentChange}
          onEmployeeStatusChange={setEmployeeStatus}
          onEmployerChange={handleEmployerChange}
          onFilterModeChange={handleFilterModeChange}
          onSearchKeyDown={(e) => {
            keyDownRef.current?.(e);
          }}
          onSearchQueryChange={setSearchQuery}
          onSpecificMonthChange={handleSpecificMonthChange}
          searchInputRef={searchInputRef}
          searchQuery={searchQuery}
          selectedDepartmentId={selectedDepartmentId}
          selectedEmployerId={selectedEmployerId}
          specificMonth={specificMonth}
          t={(key, fallback) => t(key, { defaultValue: fallback })}
        />
      </ResponsiveToolbar>

      <Suspense fallback={null}>
        <AttendanceClosurePanel
          employerId={selectedEmployerId}
          month={activeMonthYear.month}
          year={activeMonthYear.year}
        />
      </Suspense>

      {viewMode === "matrix" ? (
        <div className="h-[calc(100vh-16rem)] min-h-[500px]">
          <TimesheetMatrixGrid
            attendanceMap={attendanceMap}
            employees={matrixFilteredEmployees}
            isLoading={isLoading}
            isLoadingAttendance={isLoadingAttendance}
            month={activeMonthYear.month}
            onEmployeeClick={(emp) => {
              handleEmployeeClick(emp.id, emp.full_name);
            }}
            ref={matrixGridRef}
            year={activeMonthYear.year}
          />
        </div>
      ) : (
        <ReportsTable
          dateRange={dateRange}
          departments={departments}
          employeeStatus={employeeStatus}
          employers={employers}
          fetchNextPage={fetchNextPage}
          filteredData={filteredData}
          filterMode={filterMode}
          handleDateRangeChange={handleDateRangeChange}
          handleDepartmentChange={handleDepartmentChange}
          handleEmployeeClick={handleEmployeeClick}
          handleEmployerChange={handleEmployerChange}
          handleFilterModeChange={handleFilterModeChange}
          handleSort={handleSort}
          handleSpecificMonthChange={handleSpecificMonthChange}
          hasNextPage={hasNextPage}
          hideFilters
          isFetchingNextPage={isFetchingNextPage}
          isLoading={isLoading}
          keyDownRef={keyDownRef}
          onEmployeeStatusChange={setEmployeeStatus}
          quickStats={quickStats}
          searchInputRef={searchInputRef}
          searchQuery={searchQuery}
          selectedDepartmentId={selectedDepartmentId}
          selectedEmployerId={selectedEmployerId}
          setSearchQuery={setSearchQuery}
          sortBy={sortBy}
          specificMonth={specificMonth}
        />
      )}

      <Suspense fallback={null}>
        <EmployeeDailyDrawer dateRange={dateRange}
      employeeId={selectedEmployee?.id ?? ""}
      employeeName={selectedEmployee?.name ?? ""}
      employerId={selectedEmployee?.employerId}
      employers={clickedEmployers}
      isOpen={Boolean(selectedEmployee)}
      key={
        selectedEmployee
          ? `${selectedEmployee.id}:${selectedEmployee.employerId ?? "none"}`
          : "closed"
      }
        onClose={handleDrawerClose}  />
      </Suspense>

      <Suspense fallback={null}>
        <ReportExportDialog employeeIds={
        searchQuery && filteredData.length > 0
          ? filteredData
              .map((d) => d.employee_id)
              .filter((id): id is string => Boolean(id))
          : undefined
      }
      endDate={dateRange.end}
      isOpen={isExportDialogOpen}
      onClose={() => {
        setIsExportDialogOpen(false);
      }}
        reportType="detail"
        startDate={dateRange.start}  />
      </Suspense>
    </div>
  );
}
