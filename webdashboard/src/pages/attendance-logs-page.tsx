import React from "react";
import { useTranslation } from "react-i18next";

import FloatingRecordBadge from "@/components/common/floating-record-badge";
import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import { RecordedLogsTable } from "@/components/features/attendance/recorded-logs-table";
import { StagingLogsTable } from "@/components/features/attendance/staging-logs-table";

import { AttendanceLogsFilters } from "./attendance/components/attendance-logs-filters";
import { AttendanceLogsHeader } from "./attendance/components/attendance-logs-header";
import { AttendanceLogsTabs } from "./attendance/components/attendance-logs-tabs";
import { useAttendanceLogsPage } from "./attendance/hooks/use-attendance-logs-page";

export default function AttendanceLogsPage() {
  const { t } = useTranslation();
  const {
    wsId,
    formatDate,
    formatTime,
    activeTab,
    setActiveTab,
    employeeId,
    setEmployeeId,
    filterMode,
    dateRange,
    setDateRange,
    specificMonth,
    isImportDialogOpen,
    setIsImportDialogOpen,
    handleFilterModeChange,
    handleSpecificMonthChange,
    allRecordedLogs,
    recordedTotal,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
    allStagingLogs,
    stagingTotal,
    stagingLoading,
    stagingCount,
    isFetchingNextStagingPage,
    hasNextStagingPage,
    fetchNextStagingPage,
    getEmployeeName,
    getEmployeeID,
    getDeviceName,
    loadMoreRef,
    loadMoreStagingRef,
    handleCreateEmployee,
    dismissMutation,
    resolveMutation,
  } = useAttendanceLogsPage();

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-700 pb-20">
      <AttendanceLogsHeader
        dateRange={dateRange}
        setIsImportDialogOpen={setIsImportDialogOpen}
      />

      <AttendanceLogsTabs
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        stagingCount={stagingCount}
      />

      {activeTab === "recorded" && (
        <>
          <AttendanceLogsFilters
            dateRange={dateRange}
            employeeId={employeeId}
            filterMode={filterMode}
            handleFilterModeChange={handleFilterModeChange}
            handleSpecificMonthChange={handleSpecificMonthChange}
            setDateRange={setDateRange}
            setEmployeeId={setEmployeeId}
            specificMonth={specificMonth}
            wsId={wsId}
          />
          
          <RecordedLogsTable
            formatDate={formatDate}
            formatTime={formatTime}
            getDeviceName={getDeviceName}
            getEmployeeID={getEmployeeID}
            getEmployeeName={getEmployeeName}
            isFetchingNextPage={isFetchingNextPage}
            isLoading={isLoading}
            loadMoreRef={loadMoreRef}
            logs={allRecordedLogs}
          />
        </>
      )}

      {activeTab === "unknown" && (
        <StagingLogsTable
          formatDate={formatDate}
          formatTime={formatTime}
          getDeviceName={getDeviceName}
          isDismissing={dismissMutation.isPending}
          isFetchingNextPage={isFetchingNextStagingPage}
          isLoading={stagingLoading}
          loadMoreRef={loadMoreStagingRef}
          logs={allStagingLogs}
          onCreateEmployee={handleCreateEmployee}
          onDismiss={(logId) => { dismissMutation.mutate(logId); }}
          onResolve={(logId, empId) => { resolveMutation.mutate({ logId, employeeId: empId }); }}
          wsId={wsId}
        />
      )}

      {/* Floating record count badge */}
      {activeTab === "recorded" ? (
        <FloatingRecordBadge label={t("common.pagination.records")} loaded={allRecordedLogs.length} total={recordedTotal} />
      ) : (
        <FloatingRecordBadge label={t("common.pagination.records")} loaded={allStagingLogs.length} total={stagingTotal} variant="amber" />
      )}

      <AttendanceImportDialog
        endpoint="/attendance/logs/upload"
        isOpen={isImportDialogOpen}
        onClose={() => { setIsImportDialogOpen(false); }}
        targetType="log"
        title={t("attendance_logs.modal.import_title")}
      />
    </div>
  );
}
