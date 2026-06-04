import React from "react";
import { useTranslation } from "react-i18next";

import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import { useMyPermissions } from "@/hooks/use-permissions";

import { BehalfAssignmentsTab } from "./approvals/components/behalf-assignments-tab";
import { AdjustmentsHeader } from "./attendance/components/adjustments-header";
import { AdjustmentsTable } from "./attendance/components/adjustments-table";
import { AdjustmentsTabs } from "./attendance/components/adjustments-tabs";
import { useAttendanceAdjustmentsPage } from "./attendance/hooks/use-attendance-adjustments-page";

export default function AttendanceAdjustmentsPage() {
  const { t } = useTranslation();
  const { hasPermission } = useMyPermissions();
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

  const canManageBehalf = hasPermission("attendance:adjustments:manage_behalf");

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <AdjustmentsHeader
        onImportClick={() => { setIsImportDialogOpen(true); }}
      />

      <AdjustmentsTabs
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        canManageBehalf={canManageBehalf}
      />

      {activeTab === "behalf_assignments" && canManageBehalf ? (
        <BehalfAssignmentsTab assignmentType="attendance_adjustment" wsId={wsId} />
      ) : (
        <AdjustmentsTable
          activeTab={activeTab}
          formatDate={formatDate}
          formatTime={formatTime}
          getEmployeeID={getEmployeeID}
          getEmployeeName={getEmployeeName}
          isLoading={isLoading}
          onApprove={(id, status) => { approveMutation.mutate({ id, status: status as "approved" | "rejected" }); }}
          requests={requests}
        />
      )}

      <AttendanceImportDialog
        endpoint="/attendance-adjustments/upload"
        expectedColumns={[
          {
            key: "employee_code",
            label: t("attendance_import.columns.employee_code.label"),
            required: true,
            description: t("attendance_import.columns.employee_code.desc")
          },
          {
            key: "punch_time",
            label: t("attendance_import.columns.punch_time.label"),
            required: true,
            format: "DD/MM/YYYY HH:mm " + t("attendance_import.or") + " YYYY-MM-DD HH:mm",
            description: t("attendance_import.columns.punch_time.desc")
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