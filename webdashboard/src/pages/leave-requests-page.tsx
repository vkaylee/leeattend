import { Clock, Filter, Plus, ShieldCheck, Upload } from "lucide-react";
import React, { useMemo, useState } from "react";

import { EmployeeSearch } from "@/components/common/employee-search";
import FloatingRecordBadge from "@/components/common/floating-record-badge";
import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import { ResilientWidget } from "@/components/common/resilient-widget";
import { CreateLeaveRequestModal } from "@/components/features/leave/create-leave-request-modal";
import { LeaveImportBanner } from "@/components/features/leave/leave-import-banner";
import LeaveImportDialog from "@/components/features/leave/leave-import-dialog";
import { LeaveRequestRow } from "@/components/features/leave/leave-request-row";
import { useLeaveRequests, TabType, LeaveStatusFilter } from "@/components/features/leave/use-leave-requests";
import { PDFPreviewModal } from "@/components/files/pdf-preview-modal";
import ApprovalListLayout from "@/components/templates/approval-list-layout";
import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { PageTabs, PageTab } from "@/components/ui/page-tabs";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPermissions } from "@/hooks/use-permissions";
import { getCurrentWorkspaceId, Employee } from "@/lib/api";
import { cn } from "@/lib/utils";

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

  const canManageBehalf = hasPermission("leave:policies:manage");

  const tabs: PageTab[] = [
    {
      id: "requests",
      label: t("leave_requests.title", "Danh sách đơn").toUpperCase(),
      icon: <Clock size={14} />,
      colorScheme: "primary",
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
    <th className="py-6 text-[10px] font-bold uppercase tracking-[0.25em] text-center px-8" key="status-header">
      <div className="flex justify-center">
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                className={cn(
                  "flex items-center gap-1.5 h-auto min-w-0 transition-colors group cursor-pointer px-3 py-1.5 rounded-xl border border-transparent bg-transparent font-bold text-[10px] uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800/50",
                  statusFilter !== "all"
                    ? "text-primary-600 dark:text-primary-450 bg-primary-50/50 dark:bg-primary-950/30 border-primary-150/50 dark:border-primary-900/50 font-extrabold"
                    : "text-slate-500 dark:text-slate-400 hover:text-slate-850 dark:hover:text-slate-200"
                )}
                type="button"
              >
                <span className="flex items-center gap-1.5">
                  {t("common.status")}
                  {statusFilter !== "all" && (
                    <span className="text-[8px] font-extrabold text-primary-500 dark:text-primary-400 normal-case bg-primary-100/50 dark:bg-primary-900/40 px-1.5 py-0.5 rounded-md">
                      {t(`common.status_${statusFilter}`).toUpperCase()}
                    </span>
                  )}
                </span>
                <Filter
                  className={cn(
                    "text-slate-400 group-hover:text-slate-600 dark:group-hover:text-slate-350 transition-colors",
                    statusFilter !== "all" && "text-primary-500 dark:text-primary-400 fill-primary-500/20"
                  )}
                  size={11}
                />
              </Button>
            </DropdownMenuTrigger>
          <DropdownMenuContent align="center" className="w-52">
            <DropdownMenuLabel>{t("common.filter_by_status", "Filter by Status").toUpperCase()}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuRadioGroup onValueChange={(val) => { setStatusFilter(val as LeaveStatusFilter); }} value={statusFilter}>
              <DropdownMenuRadioItem value="all">{t("common.all", "All").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="pending">{t("common.status_pending", "Pending").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="approved">{t("common.status_approved", "Approved").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="pre_approved">{t("common.status_pre_approved", "Pre-approved").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="rejected">{t("common.status_rejected", "Rejected").toUpperCase()}</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </th>,
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

      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <PageTabs
          activeTab={activeTab}
          onTabChange={(id) => { setActiveTab(id as TabType); }}
          tabs={tabs}
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
        <ResilientWidget>
          <BehalfAssignmentsTab assignmentType="leave" wsId={wsId} />
        </ResilientWidget>
      ) : (
        <ResilientWidget>
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
                  fileUrl={previewFileUrl}
                  isOpen={isPreviewOpen}
                  onClose={() => { setIsPreviewOpen(false); }}
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
                onPreviewAttachment={(url, title) => {
                  setPreviewFileUrl(url);
                  setPreviewTitle(title);
                  setIsPreviewOpen(true);
                }}
                request={request}
                t={t}
              />
            ))}
            {!!hasNextPage && (
              <InfiniteScrollSentinel
                colSpan={tableHeaders.length}
                isFetchingNextPage={isFetchingNextPage}
                sentinelRef={loadMoreRef}
              />
            )}
          </ApprovalListLayout>
        </ResilientWidget>
      )}

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
