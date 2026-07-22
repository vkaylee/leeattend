/* eslint-disable complexity, max-lines-per-function */
import type { TFunction } from "i18next";

import { useQuery } from "@tanstack/react-query";
import { CheckCircle2, Clock, Filter, MessageSquare, Moon, Paperclip, Plus, ShieldCheck, Sun, Timer, Upload, XCircle } from "lucide-react";
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
import ResponsiveTable from "@/components/ui/responsive-table";
import ResponsiveToolbar from "@/components/ui/responsive-toolbar";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useWorkspace } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId, fetchDepartments, Employee, LeaveRequest, LeaveType } from "@/lib/api";
import { cn } from "@/lib/utils";

import { BehalfAssignmentsTab } from "./approvals/components/behalf-assignments-tab";

function LeaveRequestMobileCard({
  request,
  emp,
  leaveTypes,
  formatDate,
  t,
  onApprove,
  onPreviewAttachment,
}: {
  emp?: Employee;
  formatDate: (d: string) => string;
  leaveTypes: LeaveType[];
  onApprove: (id: string, status: string) => void;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  request: LeaveRequest;
  t: TFunction;
}) {
  const { workspace } = useWorkspace();
  const wsId = workspace?.id ?? "";
  const leaveTypeName = (leaveTypes.find(tOption => tOption.id === request.leave_type_id)?.name ?? "Unknown").toUpperCase();

  const getDurationBadge = () => {
    if (request.duration_type === "first_half") {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider border bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-200 dark:border-amber-800">
          <Sun size={8} /> {t("leave_requests.duration.first_half")}
        </span>
      );
    }
    if (request.duration_type === "second_half") {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider border bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-200 dark:border-blue-800">
          <Moon size={8} /> {t("leave_requests.duration.second_half")}
        </span>
      );
    }
    if (request.duration_type === "custom_hours") {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider border bg-cyan-50 dark:bg-cyan-950 text-cyan-600 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800">
          <Timer size={8} /> {request.start_time?.slice(0, 5) ?? ""}–{request.end_time?.slice(0, 5) ?? ""}
        </span>
      );
    }
    return null;
  };

  const getStatusBadge = () => {
    const statusStyles: Record<string, string> = {
      pending: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800",
      approved: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800",
      pre_approved: "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100 dark:border-teal-800",
      rejected: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800",
    };

    const statusLabels: Record<string, React.ReactNode> = {
      pending: t("common.status_pending").toUpperCase(),
      approved: t("common.status_approved").toUpperCase(),
      pre_approved: (
        <span className="flex items-center gap-1">
          <ShieldCheck size={10} />
          {t("common.status_pre_approved").toUpperCase()}
        </span>
      ),
      rejected: t("common.status_rejected").toUpperCase(),
    };

    return (
      <span className={cn(
        "px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm inline-block",
        statusStyles[request.status] ?? statusStyles.pending
      )}>
        {statusLabels[request.status] ?? statusLabels.pending}
      </span>
    );
  };

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Employee info */}
      <div className="flex items-start gap-3">
        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 font-bold uppercase text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400">
          {emp?.full_name[0] ?? "?"}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
            {emp?.full_name ?? "N/A"}
          </div>
          <div className="mt-1 font-mono text-[10px] font-bold uppercase tracking-wider text-slate-400">
            {emp?.employee_id_number}
          </div>
        </div>
      </div>

      {/* Leave type and dates */}
      <div className="mt-4 space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="flex items-center justify-between">
          <span className="px-3 py-1 bg-slate-100 dark:bg-slate-700 text-primary-700 dark:text-primary-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-primary-200 dark:border-primary-800/50 shadow-sm">
            {leaveTypeName}
          </span>
          {getStatusBadge()}
        </div>

        <div className="space-y-2">
          <div className="text-xs font-bold text-primary-700 dark:text-primary-300 uppercase tracking-tight">
            {formatDate(request.start_date)} — {formatDate(request.end_date)}
          </div>
          {request.duration_type !== "full_day" && getDurationBadge()}
        </div>

        {/* Reason */}
        <div className="bg-slate-50 dark:bg-slate-800/50 p-3 rounded-xl border border-primary-100 dark:border-primary-800/50 shadow-inner">
          <Typography className="text-[10px] text-primary-400 font-bold leading-relaxed line-clamp-2 italic opacity-80 flex gap-2" variant="p">
            <MessageSquare className="text-slate-300 dark:text-slate-600 mt-0.5 shrink-0" size={12} />
            &quot;{request.reason ?? t("common.no_description")}&quot;
          </Typography>
        </div>

        {/* Attachments */}
        {!!request.attachment_file_ids && request.attachment_file_ids.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {request.attachment_file_ids.map((fileId, index) => {
              if (!fileId) return null;
              return (
                <Button
                  className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[10px] font-bold bg-indigo-50 hover:bg-indigo-100 text-indigo-750 dark:bg-indigo-950/30 dark:hover:bg-indigo-900/40 dark:text-indigo-400 border border-indigo-150 dark:border-indigo-900/50 transition-colors cursor-pointer h-auto py-1 shadow-none"
                  key={fileId}
                  onClick={() => { onPreviewAttachment?.(`/api/v1/workspaces/${wsId}/files/${fileId}/download`, `Leave Attachment #${index + 1}`); }}
                  variant="ghost"
                >
                  <Paperclip className="w-3 h-3" />
                  <span>Doc #{index + 1}</span>
                </Button>
              );
            })}
          </div>
        )}
      </div>

      {/* Actions */}
      {request.status === "pending" && (
        <div className="mt-4 flex justify-end gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
          <Button
            className="border-primary-100 bg-white text-emerald-400 hover:text-emerald-600 dark:border-primary-800/50 dark:bg-slate-900 dark:text-emerald-300 h-11 w-11 p-3"
            onClick={() => { onApprove(request.id, "approved"); }}
            size="icon"
            title={t("common.approve")}
            variant="outline"
          >
            <CheckCircle2 size={20} />
          </Button>
          <Button
            className="border-primary-100 bg-white text-rose-400 hover:text-rose-600 dark:border-primary-800/50 dark:bg-slate-900 dark:text-rose-300 h-11 w-11 p-3"
            onClick={() => { onApprove(request.id, "rejected"); }}
            size="icon"
            title={t("common.reject")}
            variant="outline"
          >
            <XCircle size={20} />
          </Button>
        </div>
      )}
    </div>
  );
}

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

  const { data: departments = [] } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId ?? ""),
    enabled: !!wsId,
  });

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
                className={cn(
                  "flex items-center gap-1.5 h-auto min-w-0 transition-colors group cursor-pointer px-3 py-1.5 rounded-xl border border-transparent bg-transparent font-bold text-[10px] uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800/50",
                  statusFilter !== "all"
                    ? "text-primary-600 dark:text-primary-450 bg-primary-50/50 dark:bg-primary-950/30 border-primary-150/50 dark:border-primary-900/50 font-extrabold"
                    : "text-slate-500 dark:text-slate-400 hover:text-slate-850 dark:hover:text-slate-200"
                )}
                type="button"
                variant="ghost"
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
    <div className="space-y-6 animate-in fade-in duration-700 pb-20">
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800 pb-4 px-1">
        <Typography className="text-2xl font-bold text-slate-850 dark:text-slate-100 tracking-tight normal-case" variant="h3">{t("leave_requests.title")}</Typography>
        <div className="flex gap-3">
          <Button
            className="rounded-xl h-10 text-xs font-bold uppercase tracking-wider shadow-sm"
            onClick={() => { setIsImportModalOpen(true); }}
            variant="outline"
          >
            <Upload className="w-4 h-4 mr-2" />
            {t("leave_requests.import_excel")}
          </Button>
          <Button className="rounded-xl h-10 text-xs font-bold uppercase tracking-wider shadow-md" onClick={() => { setIsModalOpen(true); }} variant="primary">
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
              departments={departments}
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
          <div className="space-y-6">
            <LeaveImportBanner importBannerDismissed={importBannerDismissed} importJob={importJob} setImportBannerDismissed={setImportBannerDismissed} t={t} wsId={wsId} />

            <ResponsiveToolbar
              filterCount={statusFilter !== "all" ? 1 : 0}
              mobileOnly
            >
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("common.status")}
                  </label>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button className="w-full justify-between" variant="outline">
                        <span>{t(`common.status_${statusFilter}`).toUpperCase()}</span>
                        <Filter size={14} />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent className="w-52">
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
              </div>
            </ResponsiveToolbar>

            <ResponsiveTable<LeaveRequest>
              data={requests}
              emptyState={{
                title: t("leave_requests.messages.no_requests"),
                icon: <Clock className="h-10 w-10" />,
              }}
              getCardKey={(request) => request.id}
              isLoading={requestsLoading}
              renderCard={(request) => (
                <LeaveRequestMobileCard
                  emp={employeeMap.get(request.employment_id)}
                  formatDate={formatDate}
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
              )}
              renderDesktopTable={() => (
                <ApprovalListLayout
                  activeTab={activeTab}
                  emptyMessage={t("leave_requests.messages.no_requests")}
                  hideHeader
                  hideTabs
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
              )}
            />

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
          </div>
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
