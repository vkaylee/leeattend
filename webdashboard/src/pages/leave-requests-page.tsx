import { CheckCircle2, Clock, Plus, ShieldCheck, Upload, XCircle } from "lucide-react";
import React, { useMemo, useState } from "react";

import { CreateLeaveRequestModal } from "@/components/features/leave/create-leave-request-modal";
import { LeaveImportBanner } from "@/components/features/leave/leave-import-banner";
import LeaveImportDialog from "@/components/features/leave/leave-import-dialog";
import { LeaveRequestRow } from "@/components/features/leave/leave-request-row";
import { useLeaveRequests, TabType } from "@/components/features/leave/use-leave-requests";
import ApprovalListLayout from "@/components/templates/approval-list-layout";
import Button from "@/components/ui/button";
import { PageTabs, PageTab } from "@/components/ui/page-tabs";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPermissions } from "@/hooks/use-permissions";
import { getCurrentWorkspaceId, Employee } from "@/lib/api";
import { PDFPreviewModal } from "@/components/files/PDFPreviewModal";

import { BehalfAssignmentsTab } from "./approvals/components/behalf-assignments-tab";

export default function LeaveRequestsPage() {
  const wsId = getCurrentWorkspaceId();
  const { formatDate } = useDateTimeFormatter();
  const { hasPermission } = useMyPermissions();

  const [previewFileUrl, setPreviewFileUrl] = useState<string | null>(null);
  const [previewTitle, setPreviewTitle] = useState<string>("");
  const [isPreviewOpen, setIsPreviewOpen] = useState<boolean>(false);
  
  const {
    isModalOpen, setIsModalOpen,
    isImportModalOpen, setIsImportModalOpen,
    activeTab, setActiveTab,
    employees, requests, requestsLoading, leaveTypes,
    approveMutation,
    importJob,
    importBannerDismissed, setImportBannerDismissed,
    handleImportComplete,
    t
  } = useLeaveRequests(wsId);

  const employeeMap = useMemo(
    () => {
      const map = new Map<string, Employee>();
      employees.forEach(e => {
        if (e.active_employment_id) {
          map.set(e.active_employment_id, e);
        }
        map.set(e.id, e);
      });
      return map;
    },
    [employees]
  );

  if (!wsId) return <div className="p-12 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  const canManageBehalf = hasPermission("leave:requests:manage_behalf");

  const tabs: PageTab[] = [
    {
      id: "pending",
      label: t("leave_requests.tabs.pending").toUpperCase(),
      icon: <Clock size={14} />,
      colorScheme: "primary",
    },
    {
      id: "approved",
      label: t("leave_requests.tabs.approved").toUpperCase(),
      icon: <CheckCircle2 size={14} />,
      colorScheme: "amber",
    },
    {
      id: "rejected",
      label: t("leave_requests.tabs.rejected").toUpperCase(),
      icon: <XCircle size={14} />,
      colorScheme: "rose",
    },
    {
      id: "behalf_assignments",
      label: t("leave_requests.tabs.behalf_assignments", "Behalf Assignments").toUpperCase(),
      icon: <ShieldCheck size={14} />,
      colorScheme: "violet",
      hidden: !canManageBehalf,
    },
  ];

  const tableHeaders = [
    t("common.employee"),
    t("common.type"),
    t("leave_requests.table.details"),
    t("common.status"),
    "",
  ];

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800 pb-6 px-1">
        <Typography className="text-3xl font-extrabold text-slate-850 dark:text-slate-100 tracking-tight uppercase" variant="h1">{t("leave_requests.title")}</Typography>
        <div className="flex gap-4">
          <Button
            className="rounded-2xl h-12 text-xs font-bold uppercase tracking-wider shadow-sm"
            onClick={() => { setIsImportModalOpen(true); }}
            variant="outline"
          >
            <Upload className="w-4 h-4 mr-2" />
            {t("leave_requests.import_excel")}
          </Button>
          <Button className="rounded-2xl h-12 text-xs font-bold uppercase tracking-wider shadow-lg" onClick={() => { setIsModalOpen(true); }} variant="primary">
            <Plus className="w-4 h-4 mr-2" />
            {t("leave_requests.add")}
          </Button>
        </div>
      </div>

      <PageTabs
        activeTab={activeTab}
        onTabChange={(id) => { setActiveTab(id as TabType); }}
        tabs={tabs}
      />

      {activeTab === "behalf_assignments" && canManageBehalf ? (
        <BehalfAssignmentsTab assignmentType="leave" wsId={wsId} />
      ) : (
        <ApprovalListLayout
          activeTab={activeTab}
          banner={<LeaveImportBanner importBannerDismissed={importBannerDismissed} importJob={importJob} setImportBannerDismissed={setImportBannerDismissed} t={t} wsId={wsId} />}
          emptyMessage={t("leave_requests.messages.no_requests")}
          hideHeader
          hideTabs
          isEmpty={requests.length === 0}
          isLoading={requestsLoading}
          modals={
            <>
              <CreateLeaveRequestModal isOpen={isModalOpen} leaveTypes={leaveTypes} onClose={() => { setIsModalOpen(false); }} t={t} wsId={wsId} />
              <LeaveImportDialog
                isOpen={isImportModalOpen}
                onClose={() => { setIsImportModalOpen(false); }}
                onImportComplete={handleImportComplete}
              />
              <PDFPreviewModal
                isOpen={isPreviewOpen}
                onClose={() => setIsPreviewOpen(false)}
                fileUrl={previewFileUrl}
                title={previewTitle}
              />
            </>
          }
          onTabChange={(id) => { setActiveTab(id as TabType); }}
          tableHeaders={tableHeaders}
        >
          {requests.map((request) => (
            <LeaveRequestRow
              emp={employeeMap.get(request.employment_id)}
              formatDate={formatDate}
              key={request.id}
              leaveTypes={leaveTypes}
              onApprove={(id, status) => { approveMutation.mutate({ id, status: status as "approved" | "rejected" | "pre_approved" | "pending" | "cancelled" }); }}
              request={request}
              t={t}
              onPreviewAttachment={(url, title) => {
                setPreviewFileUrl(url);
                setPreviewTitle(title);
                setIsPreviewOpen(true);
              }}
            />
          ))}
        </ApprovalListLayout>
      )}
    </div>
  );
}
