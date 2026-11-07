
import { useDepartments } from "@/hooks/use-departments";

import { useQuery } from "@tanstack/react-query";
import { Clock, Plus, ShieldCheck, Upload } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import FloatingRecordBadge from "@/components/common/floating-record-badge";
import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import { ResilientWidget } from "@/components/common/resilient-widget";
import { CreateLeaveRequestModal } from "@/components/features/leave/create-leave-request-modal";
import { LeaveImportBanner } from "@/components/features/leave/leave-import-banner";
import LeaveImportDialog from "@/components/features/leave/leave-import-dialog";
import { LeaveRequestMobileCard } from "@/components/features/leave/leave-request-mobile-card";
 import { LeaveRequestRow } from "@/components/features/leave/leave-request-row";
import { LeaveDetailSheet } from "@/components/features/leave/leave-detail-sheet";
 import { LeaveStatusFilterHeader } from "@/components/features/leave/leave-status-filter-header";
 import { useLeaveRequests, TabType, LeaveStatusFilter } from "@/components/features/leave/use-leave-requests";
import { PDFPreviewModal } from "@/components/files/pdf-preview-modal";
import ApprovalListLayout from "@/components/templates/approval-list-layout";
import Button from "@/components/ui/button";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";
import { PageTabs, PageTab } from "@/components/ui/page-tabs";
import ResponsiveTable from "@/components/ui/responsive-table";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPermissions } from "@/hooks/use-permissions";
import { getStandaloneOrganizationId, Employee, LeaveRequest } from "@/lib/api";

import { BehalfAssignmentsTab } from "./approvals/components/behalf-assignments-tab";

function StatusFilterDropdown({ setStatusFilter, statusFilter }: { setStatusFilter: (v: LeaveStatusFilter) => void; statusFilter: LeaveStatusFilter }) {
  const { t: tLocal } = useTranslation();
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{tLocal("common.status")}</label>
      <FilterPanel
        container="dropdown"
        mode="single"
        onClear={() => { setStatusFilter("all"); }}
        onToggle={(val) => { setStatusFilter(val as LeaveStatusFilter); }}
        options={[
          { value: "all", label: tLocal("common.all", "All").toUpperCase() },
          { value: "pending", label: tLocal("common.status_pending", "Pending").toUpperCase() },
          { value: "approved", label: tLocal("common.status_approved", "Approved").toUpperCase() },
          { value: "pre_approved", label: tLocal("common.status_pre_approved", "Pre-approved").toUpperCase() },
          { value: "rejected", label: tLocal("common.status_rejected", "Rejected").toUpperCase() },
        ]}
        selected={statusFilter !== "all" ? [statusFilter] : []}
        trigger={
          <FilterTrigger
            chevron
            hasActive={statusFilter !== "all"}
            label={tLocal(`common.status_${statusFilter}`).toUpperCase()}
            variant="pill-button"
          />
        }
      />
    </div>
  );
}

export default function LeaveRequestsPage() {
  const wsId = getStandaloneOrganizationId();
  const { formatDate } = useDateTimeFormatter();
  const { hasPermission } = useMyPermissions();

  const [previewFileUrl, setPreviewFileUrl] = useState<string | null>(null);
  const [previewTitle, setPreviewTitle] = useState<string>("");
   const [isPreviewOpen, setIsPreviewOpen] = useState<boolean>(false);
  const [selectedRequest, setSelectedRequest] = useState<LeaveRequest | null>(null);
 
   const {
    isModalOpen, setIsModalOpen,
    isImportModalOpen, setIsImportModalOpen,
    activeTab, setActiveTab,
    statusFilter, setStatusFilter,
    employeeId, setEmployeeId,
    employees, requests, requestsLoading, requestsTotal, leaveTypes,
    hasNextPage, isFetchingNextPage, loadMoreRef,
    approveMutation,
    importJob,
    importBannerDismissed, setImportBannerDismissed,
    handleImportComplete,
    t
  } = useLeaveRequests(wsId);

  const { data: departments = [] } = useDepartments(wsId);

  const employeeMap = useMemo(() => {
    const map = new Map<string, Employee>();
    employees.forEach(e => { if (e.active_employment_id) map.set(e.active_employment_id, e); map.set(e.id, e); });
    return map;
  }, [employees]);

  if (!wsId) return <div className="p-12 text-center text-primary-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  const canManageBehalf = hasPermission("leave:policies:manage");

  const tabs: PageTab[] = [
    { id: "requests", label: t("leave_requests.title", "Danh sách đơn").toUpperCase(), icon: <Clock size={14} />, colorScheme: "primary" },
    { id: "behalf_assignments", label: t("leave_requests.tabs.behalf_assignments", "Behalf Assignments").toUpperCase(), icon: <ShieldCheck size={14} />, colorScheme: "violet", hidden: !canManageBehalf },
  ];

  const tableHeaders = [
    t("common.employee"),
    t("common.type"),
    t("leave_requests.table.details"),
    <LeaveStatusFilterHeader key="status-header" setStatusFilter={setStatusFilter} statusFilter={statusFilter} />,
    "",
  ];

  const handleApprove = (id: string, status: string) => {
    approveMutation.mutate({ id, status: status as "approved" | "rejected" | "pre_approved" | "pending" | "cancelled" });
  };

  const handlePreview = (url: string, title: string) => {
    setPreviewFileUrl(url);
    setPreviewTitle(title);
    setIsPreviewOpen(true);
  };

  return (
    <PageContainer>
      <PageHeader
        actions={
          <div className="flex w-full flex-col gap-3 sm:w-auto sm:flex-row">
            <Button className="h-10 w-full rounded-xl text-xs font-bold uppercase tracking-wider shadow-sm sm:w-auto" onClick={() => { setIsImportModalOpen(true); }} variant="outline">
              <Upload className="w-4 h-4 mr-2" />{t("leave_requests.import_excel")}
            </Button>
            <Button className="h-10 w-full rounded-xl text-xs font-bold uppercase tracking-wider shadow-md sm:w-auto" onClick={() => { setIsModalOpen(true); }} variant="primary">
              <Plus className="w-4 h-4 mr-2" />{t("leave_requests.add")}
            </Button>
          </div>
        }
        className="border-b border-slate-50 pb-4 px-1 dark:border-slate-800"
        title={t("leave_requests.title")}
      />

      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <PageTabs activeTab={activeTab} onTabChange={(id) => { setActiveTab(id as TabType); }} tabs={tabs} />
        {activeTab !== "behalf_assignments" && (
          <div className="w-full sm:w-72">
            <EmployeeSearch
              clearOnEscape
              departments={departments}
              keyboardShortcut={["/", "Enter"]}
              onChange={(value) => { setEmployeeId((Array.isArray(value) ? value[0] : value) || "all"); }}
              placeholder={employeeId === "all" ? t("common.all_employees") : undefined}
              value={employeeId === "all" ? "" : employeeId}
              workspaceId={wsId}
            />
          </div>
        )}
      </div>

      {activeTab === "behalf_assignments" && canManageBehalf ? (
        <ResilientWidget>
          <BehalfAssignmentsTab assignmentType="leave" wsId={wsId} />
        </ResilientWidget>
      ) : (
        <ResilientWidget>
          <div className="space-y-6">
            <LeaveImportBanner importBannerDismissed={importBannerDismissed} importJob={importJob} setImportBannerDismissed={setImportBannerDismissed} t={t} wsId={wsId} />

            <ResponsiveToolbar filterCount={statusFilter !== "all" ? 1 : 0} mobileOnly>
              <div className="space-y-4">
                <StatusFilterDropdown setStatusFilter={setStatusFilter} statusFilter={statusFilter} />
              </div>
            </ResponsiveToolbar>

            <ResponsiveTable<LeaveRequest>
              data={requests}
              emptyState={{ title: t("leave_requests.messages.no_requests"), icon: <Clock className="h-10 w-10" /> }}
              getCardKey={(request) => request.id}
              isLoading={requestsLoading}
              renderCard={(request) => (
                <LeaveRequestMobileCard
                  emp={employeeMap.get(request.employment_id)}
                  formatDate={formatDate}
                  leaveTypes={leaveTypes}
                  onApprove={handleApprove}
                  onPreviewAttachment={handlePreview}
                  request={request}
                  t={t}
                />
              )}
              renderDesktopTable={() => (
                <ApprovalListLayout
                  activeTab={activeTab}
                  emptyMessage={t("leave_requests.messages.no_requests")}
                  hideHeader hideTabs
                  isEmpty={requests.length === 0}
                  isLoading={requestsLoading}
                  onTabChange={(id) => { setActiveTab(id as TabType); }}
                  tableHeaders={tableHeaders}
                >
                  {requests.map((request) => (
                    <LeaveRequestRow
                      emp={employeeMap.get(request.employment_id)}
                      formatDate={formatDate}
                      key={request.id}
                      leaveTypes={leaveTypes}
                      onApprove={handleApprove}
                      onPreviewAttachment={handlePreview}
                      onSelect={(req) => setSelectedRequest(req)}
                      request={request}
                      t={t}
                    />
                  ))}
                  {hasNextPage ? (
                    <InfiniteScrollSentinel colSpan={tableHeaders.length} isFetchingNextPage={isFetchingNextPage} sentinelRef={loadMoreRef} />
                  ) : null}
                </ApprovalListLayout>
              )}
            />

            <LeaveDetailSheet
              emp={selectedRequest ? employeeMap.get(selectedRequest.employment_id) : undefined}
              formatDate={formatDate}
              isOpen={Boolean(selectedRequest)}
              leaveTypes={leaveTypes}
              onApprove={handleApprove}
              onClose={() => setSelectedRequest(null)}
              request={selectedRequest}
            />
             <CreateLeaveRequestModal isOpen={isModalOpen} leaveTypes={leaveTypes} onClose={() => { setIsModalOpen(false); }} t={t} wsId={wsId} />
            <PDFPreviewModal fileUrl={previewFileUrl} isOpen={isPreviewOpen} onClose={() => { setIsPreviewOpen(false); }} title={previewTitle} />
          </div>
        </ResilientWidget>
      )}

      {activeTab !== "behalf_assignments" && (
        <FloatingRecordBadge label={t("common.pagination.records")} loaded={requests.length} total={requestsTotal} />
      )}
    </PageContainer>
  );
}
