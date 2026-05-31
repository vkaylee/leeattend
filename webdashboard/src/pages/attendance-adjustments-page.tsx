import React from "react";
import { useTranslation } from "react-i18next";

import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";

import { AdjustmentsHeader } from "./attendance/components/adjustments-header";
import { AdjustmentsTable } from "./attendance/components/adjustments-table";
import { AdjustmentsTabs } from "./attendance/components/adjustments-tabs";
import { useAttendanceAdjustmentsPage } from "./attendance/hooks/use-attendance-adjustments-page";

export default function AttendanceAdjustmentsPage() {
  const { t } = useTranslation();
  const {
    wsId,
    formatDate,
    formatTime,
    requests,
    isLoading,
    activeTab,
    setActiveTab,
    isImportDialogOpen,
    setIsImportDialogOpen,
    approveMutation,
    downloadTemplate,
    getEmployeeName,
    getEmployeeID,
  } = useAttendanceAdjustmentsPage();

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <AdjustmentsHeader
        onImportClick={() => { setIsImportDialogOpen(true); }}
      />

      <AdjustmentsTabs
        activeTab={activeTab}
        setActiveTab={setActiveTab}
      />

      <AdjustmentsTable
        activeTab={activeTab}
        formatDate={formatDate}
        formatTime={formatTime}
        getEmployeeID={getEmployeeID}
        getEmployeeName={getEmployeeName}
        isLoading={isLoading}
        onApprove={(id, status) => { approveMutation.mutate({ id, status }); }}
        requests={requests}
      />

      <AttendanceImportDialog
        endpoint="/attendance-adjustments/upload"
        expectedColumns={[
          {
            key: "employee_code",
            label: "Mã nhân viên",
            required: true,
            description: "Mã số định danh duy nhất của nhân viên trong hệ thống (Employee ID)."
          },
          {
            key: "punch_time",
            label: "Thời gian chấm công",
            required: true,
            format: "DD/MM/YYYY HH:mm hoặc YYYY-MM-DD HH:mm",
            description: "Mốc thời gian vào/ra ca cần điều chỉnh."
          }
        ]}
        isOpen={isImportDialogOpen}
        onClose={() => { setIsImportDialogOpen(false); }}
        onDownloadTemplate={downloadTemplate}
        targetType="adjustment"
        title={t("attendance_confirm.import_excel")}
      />
    </div>
  );
}