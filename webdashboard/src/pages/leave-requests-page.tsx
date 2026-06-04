import { CheckCircle2, Clock, Plus, ShieldCheck, Upload, XCircle } from "lucide-react";
import React, { useMemo } from "react";

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
import { getCurrentWorkspaceId } from "@/lib/api";

import { BehalfAssignmentsTab } from "./approvals/components/behalf-assignments-tab";

export default function LeaveRequestsPage() {
  const wsId = getCurrentWorkspaceId();
  const { formatDate } = useDateTimeFormatter();
  const { hasPermission } = useMyPermissions();
  
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
    () => new Map(employees.map(e => [e.id, e])),
    [employees]
  );

  if (!wsId) return <div className="p-12 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  const canManageBehalf = hasPermission("leave:policies:manage");

  const tabs: PageTab[] = [
    { id: "pending", label: t("common.status_pending"), icon: <Clock size={14} />, colorScheme: "amber" },
    { id: "approved", label: t("common.status_approved"), icon: <CheckCircle2 size={14} />, colorScheme: "emerald" },
    { id: "rejected", label: t("common.status_rejected"), icon: <XCircle size={14} />, colorScheme: "rose" },
    { id: "behalf_assignments", label: t("ot_approvals.tabs.behalf_assignments", { defaultValue: "Behalf" }), icon: <ShieldCheck size={14} />, colorScheme: "violet", hidden: !canManageBehalf },
  ];

  const tableHeaders = [
    <th className="px-10 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]" key="emp">{t("common.employee")}</th>,
    <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]" key="type">{t("common.type")}</th>,
    <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em]" key="time">{t("common.time")}</th>,
    <th className="px-8 py-6 text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em] text-center" key="status">{t("common.status")}</th>,
    <th className="px-10 py-6 border-b border-primary-100 dark:border-primary-800/50" key="actions"></th>,
  ];

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row justify-between items-center gap-8 px-2">
        <div className="space-y-1">
          <Typography className="text-5xl font-bold tracking-tighter text-primary-700 dark:text-primary-300 uppercase leading-none" variant="h1">
            {t("leave_requests.title")}
          </Typography>
          <Typography className="text-[10px] font-bold text-primary-400 uppercase tracking-[0.25em] opacity-60" variant="p">
            {t("leave_requests.subtitle")}
          </Typography>
        </div>
        <div className="flex items-center gap-3 flex-wrap">
          {activeTab !== "behalf_assignments" && (
            <>
              <Button className="bg-emerald-600 hover:bg-emerald-700 shadow-emerald-100 dark:shadow-black/30 text-[10px] px-5 py-3"
                disabled={importJob?.status === "processing"}
                onClick={() => { setIsImportModalOpen(true); }}
                variant="ghost"
              >
                <Upload size={16} />
                {t("leave_requests.upload_btn", "Upload Excel")}
              </Button>
              <Button onClick={() => { setIsModalOpen(true); }} variant="ghost">
                <Plus size={18} /> {t("leave_requests.create_btn").toUpperCase()}
              </Button>
            </>
          )}
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
            </>
          }
          onTabChange={(id) => { setActiveTab(id as TabType); }}
          tableHeaders={tableHeaders}
        >
          {requests.map((request) => (
            <LeaveRequestRow
              emp={employeeMap.get(request.employee_id)}
              formatDate={formatDate}
              key={request.id}
              leaveTypes={leaveTypes}
              onApprove={(id, status) => { approveMutation.mutate({ id, status: status as "approved" | "rejected" | "pre_approved" | "pending" | "cancelled" }); }}
              request={request}
              t={t}
            />
          ))}
        </ApprovalListLayout>
      )}
    </div>
  );
}
