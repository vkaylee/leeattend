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

type AttendanceLogsState = ReturnType<typeof useAttendanceLogsPage>;

function TabContent({ state }: { state: AttendanceLogsState }) {
  const {
    activeTab, 
    dateRange, 
    employeeId, 
    filterMode, 
    handleFilterModeChange, 
    handleSpecificMonthChange, 
    setDateRange, 
    setEmployeeId, 
    specificMonth, 
    wsId,
    formatDate,
    formatTime,
    getDeviceName,
    getEmployeeID,
    getEmployeeName,
    isFetchingNextPage,
    isLoading,
    loadMoreRef,
    allRecordedLogs,
    allStagingLogs,
    stagingLoading,
    isFetchingNextStagingPage,
    loadMoreStagingRef,
    handleCreateEmployee,
    dismissMutation,
    resolveMutation
  } = state;

  if (!wsId) return null;

  if (activeTab === "recorded") {
    return (
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
    );
  }

  return (
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
      onDismiss={(logId: string) => { dismissMutation.mutate(logId); }}
      onResolve={(logId: string, empId: string) => { resolveMutation.mutate({ logId, employeeId: empId }); }}
      wsId={wsId}
    />
  );
}

export default function AttendanceLogsPage() {
  const { t } = useTranslation();
  const state = useAttendanceLogsPage();
  const { wsId, activeTab, stagingCount, allRecordedLogs, recordedTotal, allStagingLogs, stagingTotal, isImportDialogOpen, setIsImportDialogOpen } = state;

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-700 pb-20">
      <AttendanceLogsHeader dateRange={state.dateRange} setIsImportDialogOpen={setIsImportDialogOpen} />
      <AttendanceLogsTabs activeTab={activeTab} setActiveTab={state.setActiveTab} stagingCount={stagingCount} />
      <TabContent state={state} />
      <FloatingRecordBadge 
        label={t("common.pagination.records")} 
        loaded={activeTab === "recorded" ? allRecordedLogs.length : allStagingLogs.length} 
        total={activeTab === "recorded" ? recordedTotal : stagingTotal} 
        variant={activeTab === "recorded" ? undefined : "amber"} 
      />
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
