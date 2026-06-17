import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import FloatingRecordBadge from "@/components/common/floating-record-badge";
import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import { CreateAdjustmentModal } from "@/components/features/attendance/create-adjustment-modal";
import { PDFPreviewModal } from "@/components/files/PDFPreviewModal";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useWorkspace } from "@/hooks/use-workspace";

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
    statusFilter,
    setStatusFilter,
    employeeId,
    setEmployeeId,
    requestsTotal,
    hasNextPage,
    isFetchingNextPage,
    loadMoreRef,
    isImportDialogOpen,
    setIsImportDialogOpen,
    approveMutation,
    downloadTemplate,
    getEmployeeName,
    getEmployeeID,
  } = useAttendanceAdjustmentsPage();

  const { workspace } = useWorkspace(wsId ?? undefined);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

  const [previewFileUrl, setPreviewFileUrl] = useState<string | null>(null);
  const [previewTitle, setPreviewTitle] = useState<string>("");
  const [isPreviewOpen, setIsPreviewOpen] = useState(false);

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  const canManageBehalf = hasPermission("attendance:adjustments:manage_behalf");
  const canApprove = hasPermission("attendance:adjustments:approve");
  const canSubmit = hasPermission("attendance:adjustments:submit_on_behalf");

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <AdjustmentsHeader
        canSubmit={canSubmit}
        onImportClick={() => { setIsImportDialogOpen(true); }}
        onRequestClick={() => { setIsCreateModalOpen(true); }}
      />

      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <AdjustmentsTabs
          activeTab={activeTab}
          canManageBehalf={canManageBehalf}
          setActiveTab={setActiveTab}
        />
        {activeTab !== "behalf_assignments" && (
          <div className="w-full sm:w-72">
            <EmployeeSearch
              clearOnEscape
              keyboardShortcut={["/", "Enter"]}
              onChange={(value) => {
                setEmployeeId((Array.isArray(value) ? value[0] : value) || "all");
              }}
              placeholder={employeeId === "all" ? t("common.all_employees") : undefined}
              value={employeeId === "all" ? "" : employeeId}
              workspaceId={wsId}
            />
          </div>
        )}
      </div>

      {activeTab === "behalf_assignments" && canManageBehalf ? (
        <BehalfAssignmentsTab assignmentType="attendance_adjustment" wsId={wsId} />
      ) : (
        <AdjustmentsTable
          activeTab={activeTab}
          canApprove={canApprove}
          formatDate={formatDate}
          formatTime={formatTime}
          getEmployeeID={getEmployeeID}
          getEmployeeName={getEmployeeName}
          hasNextPage={hasNextPage}
          isFetchingNextPage={isFetchingNextPage}
          isLoading={isLoading}
          loadMoreRef={loadMoreRef}
          onApprove={(id, status) => { approveMutation.mutate({ id, status: status as "approved" | "rejected" }); }}
          onPreviewAttachment={(url, title) => {
            setPreviewFileUrl(url);
            setPreviewTitle(title);
            setIsPreviewOpen(true);
          }}
          requests={requests}
          setStatusFilter={setStatusFilter}
          statusFilter={statusFilter}
        />
      )}

      <CreateAdjustmentModal
        isOpen={isCreateModalOpen}
        onClose={() => { setIsCreateModalOpen(false); }}
        timezone={workspace?.timezone ?? "UTC"}
        wsId={wsId}
      />

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

      <PDFPreviewModal
        fileUrl={previewFileUrl}
        isOpen={isPreviewOpen}
        onClose={() => { setIsPreviewOpen(false); }}
        title={previewTitle}
      />

      {activeTab !== "behalf_assignments" && (
        <FloatingRecordBadge 
          label={t("common.pagination.records")} 
          loaded={requests.length} 
          total={requestsTotal} 
        />
      )}
    </div>
  );
}