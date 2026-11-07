import React from "react";
import { useTranslation } from "react-i18next";

import {
  TimesheetMatrixGrid,
  type TimesheetMatrixGridHandle,
} from "@/components/features/attendance/timesheet-matrix-grid";
import EmployeeDailyDrawer from "@/components/features/reports/employee-daily-drawer";
import ReportExportDialog from "@/components/features/reports/report-export-dialog";
import ReportFilters from "@/components/features/reports/report-filters";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";

import { ReportsHeader } from "./reports/components/reports-header";
import { ReportsTable } from "./reports/components/reports-table";
import { useMonthlyTimesheetData } from "./reports/hooks/use-monthly-timesheet-data";
import { useReportsPage } from "./reports/hooks/use-reports-page";

// eslint-disable-next-line complexity
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
    wsId,
  } = useReportsPage();

  const matrixGridRef = React.useRef<TimesheetMatrixGridHandle>(null);

  const { attendanceMap, isLoading: isLoadingAttendance } = useMonthlyTimesheetData({
    wsId,
    month: specificMonth.month,
    year: specificMonth.year,
    enabled: viewMode === "matrix",
  });

  // Filter employees for TimesheetMatrixGrid using shared active filter state
  const matrixFilteredEmployees = React.useMemo(() => {
    let result = allEmployees;

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (emp) =>
          emp.full_name?.toLowerCase().includes(q) ||
          emp.employee_code?.toLowerCase().includes(q) ||
          emp.department?.toLowerCase().includes(q)
      );
    }

    if (selectedDepartmentId) {
      result = result.filter((emp) => emp.department_id === selectedDepartmentId);
    }

    if (selectedEmployerId) {
      result = result.filter((emp) => emp.employer_id === selectedEmployerId);
    }

    if (employeeStatus === "active") {
      result = result.filter((emp) => emp.active !== false);
    } else if (employeeStatus === "inactive") {
      result = result.filter((emp) => emp.active === false);
    }

    return result;
  }, [allEmployees, searchQuery, selectedDepartmentId, selectedEmployerId, employeeStatus]);

  // Context-aware export trigger: CSV for Matrix, Dialog for Summary
  const handleExport = React.useCallback(() => {
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

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

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
            filterMode !== "this_month",
            employeeStatus !== "active",
            selectedDepartmentId !== null,
            selectedEmployerId !== null,
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
          t={t}
        />
      </ResponsiveToolbar>

      {viewMode === "matrix" ? (
        <div className="h-[calc(100vh-16rem)] min-h-[500px]">
          <TimesheetMatrixGrid
            attendanceMap={attendanceMap}
            employees={matrixFilteredEmployees}
            isLoading={isLoading}
            isLoadingAttendance={isLoadingAttendance}
            month={specificMonth.month}
            onEmployeeClick={(emp) => {
              handleEmployeeClick(emp.id, emp.full_name);
            }}
            ref={matrixGridRef}
            year={specificMonth.year}
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

      <EmployeeDailyDrawer
        dateRange={dateRange}
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
        onClose={handleDrawerClose}
        workspaceId={wsId}
      />

      <ReportExportDialog
        employeeIds={
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
        startDate={dateRange.start}
        wsId={wsId}
      />
    </div>
  );
}
