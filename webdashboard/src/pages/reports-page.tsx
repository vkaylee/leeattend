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
  } = useReportsPage();

  const matrixGridRef = React.useRef<TimesheetMatrixGridHandle>(null);

  const startDate = dateRange.start;
  // Active month & year for matrix should be synchronized with dateRange and specificMonth!
  const activeMonthYear = React.useMemo(() => {
    if (filterMode === "specific_month") {
      return specificMonth;
    }
    if (startDate) {
      const parts = startDate.split("-");
      if (parts.length >= 2) {
        const y = parseInt(parts[0], 10);
        const m = parseInt(parts[1], 10);
        if (!isNaN(y) && !isNaN(m) && m >= 1 && m <= 12) {
          return { month: m, year: y };
        }
      }
    }
    return specificMonth;
  }, [filterMode, startDate, specificMonth]);

  const { attendanceMap, isLoading: isLoadingAttendance } = useMonthlyTimesheetData({
    month: activeMonthYear.month,
    year: activeMonthYear.year,
    enabled: viewMode === "matrix",
  });

  // Filter employees for TimesheetMatrixGrid using shared active filter state
  const matrixFilteredEmployees = React.useMemo(() => {
    let result = allEmployees;

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (emp) =>
          emp.full_name.toLowerCase().includes(q) ||
          emp.employee_id_number.toLowerCase().includes(q) ||
          emp.department_name?.toLowerCase().includes(q) ||
          (emp as { employee_code?: string }).employee_code?.toLowerCase().includes(q) ||
          (emp as { department?: string }).department?.toLowerCase().includes(q)
      );
    }

    if (selectedDepartmentId) {
      result = result.filter((emp) => emp.department_name === selectedDepartmentId);
    }

    if (selectedEmployerId) {
      result = result.filter((emp) => emp.employer_id === selectedEmployerId);
    }

    if (employeeStatus === "active") {
      result = result.filter((emp) => emp.active_employment_id !== null);
    } else if (employeeStatus === "inactive") {
      result = result.filter((emp) => emp.active_employment_id === null);
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
    </div>
  );
}
