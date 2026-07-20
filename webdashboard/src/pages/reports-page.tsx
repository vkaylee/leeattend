import React from "react";
import { useTranslation } from "react-i18next";

import EmployeeDailyDrawer from "@/components/features/reports/employee-daily-drawer";
import ReportExportDialog from "@/components/features/reports/report-export-dialog";

import { ReportsHeader } from "./reports/components/reports-header";
import { ReportsTable } from "./reports/components/reports-table";
import { useReportsPage } from "./reports/hooks/use-reports-page";

  // eslint-disable-next-line complexity
export default function ReportsPage() {
  const { t } = useTranslation();
  const {
    wsId,
    dateRange,
    filterMode,
    specificMonth,
    searchQuery,
    setSearchQuery,
    isExportDialogOpen,
    setIsExportDialogOpen,
    selectedDepartmentId,
    selectedEmployerId,
    selectedEmployee,
    sortBy,
    employeeStatus,
    setEmployeeStatus,
    departments,
    employers,
    quickStats,
    filteredData,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    searchInputRef,
    keyDownRef,
    handleFilterModeChange,
    handleSpecificMonthChange,
    handleDateRangeChange,
    handleDepartmentChange,
    handleEmployerChange,
    handleSort,
    handleEmployeeClick,
    handleDrawerClose,
  } = useReportsPage();

  const clickedEmployeeSummaryRows = selectedEmployee?.id
    ? filteredData.filter(row => row.employee_id === selectedEmployee.id)
    : [];

  const clickedEmployers = clickedEmployeeSummaryRows.map(row => ({
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
        onExportClick={() => { setIsExportDialogOpen(true); }}
      />

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

      <EmployeeDailyDrawer
        dateRange={dateRange}
        employeeId={selectedEmployee?.id ?? ""}
        employeeName={selectedEmployee?.name ?? ""}
        employerId={selectedEmployee?.employerId}
        employers={clickedEmployers}
        isOpen={!!selectedEmployee}
        key={selectedEmployee ? `${selectedEmployee.id}:${selectedEmployee.employerId ?? "none"}` : "closed"}
        onClose={handleDrawerClose}
        workspaceId={wsId}
      />

      <ReportExportDialog
        employeeIds={searchQuery && filteredData.length > 0 ? filteredData.map(d => d.employee_id).filter((id): id is string => !!id) : undefined}
        endDate={dateRange.end}
        isOpen={isExportDialogOpen}
        onClose={() => { setIsExportDialogOpen(false); }}
        reportType="detail"
        startDate={dateRange.start}
        wsId={wsId}
      />
    </div>
  );
}
