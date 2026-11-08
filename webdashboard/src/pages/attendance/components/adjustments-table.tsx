import { Check, Clock, Paperclip, User as UserIcon, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { useOrganization } from "@/hooks/use-organization";
import { AttendanceAdjustmentRequest } from "@/lib/api/types";

import type { AttendanceStatusFilter } from "../hooks/use-attendance-adjustments-page";

import { AdjustmentsTableEmpty } from "./adjustments-table/empty-state";
import { AdjustmentsTableHeader } from "./adjustments-table/header";
import { AdjustmentsTableLoading } from "./adjustments-table/loading-state";
import { AdjustmentsTableRow } from "./adjustments-table/row";

function AttachmentList({
  fileIds,
  onPreviewAttachment,
}: {
  fileIds: (string | null)[];
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
}) {
  return (
    <div className="flex flex-wrap gap-2">
      {fileIds.map((fileId, index) => {
        if (!fileId) return null;
        return (
          <Button
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[10px] font-bold bg-indigo-50 hover:bg-indigo-100 text-indigo-750 dark:bg-indigo-950/30 dark:hover:bg-indigo-900/40 dark:text-indigo-400 border border-indigo-150 dark:border-indigo-900/50 transition-colors cursor-pointer"
            icon={<Paperclip className="w-3 h-3" />}
            key={fileId}
            onClick={() => { onPreviewAttachment?.(`/api/v1/files/${fileId}/download`, `Adjustment Attachment #${index + 1}`); }}
            type="button"
          >
            Doc #{index + 1}
          </Button>
        );
      })}
    </div>
  );
}

// eslint-disable-next-line complexity -- remaining branches are JSX conditional rendering + status badge variant switch
function AdjustmentMobileCard({
  activeTab,
  canApprove,
  formatDate,
  formatTime,
  getEmployeeID,
  getEmployeeName,
  onApprove,
  onPreviewAttachment,
  request,
}: {
  activeTab: string;
  canApprove?: boolean;
  formatDate: (date: string) => string;
  formatTime: (date: string) => string;
  getEmployeeID: (id: string) => string;
  getEmployeeName: (id: string) => string;
  onApprove: (id: string, status: string) => void;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  request: AttendanceAdjustmentRequest;
}) {
  const { t } = useTranslation();

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Employee info */}
      <div className="flex items-start gap-3">
        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400">
          <UserIcon size={20} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
            {getEmployeeName(request.employment_id)}
          </div>
          <div className="mt-1 font-mono text-[10px] font-bold uppercase tracking-wider text-slate-400">
            ID: {getEmployeeID(request.employment_id)}
          </div>
        </div>
        <StatusBadge
          variant={
            request.status === "pending" ? "warning" : request.status === "approved" ? "success" : "danger"
          }
        >
          {t(`common.status_${request.status}`).toUpperCase()}
        </StatusBadge>
      </div>

      {/* Date/Time and reason */}
      <div className="mt-4 space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="inline-flex flex-col items-center rounded-xl border border-slate-100 bg-slate-50 px-4 py-2 shadow-inner dark:border-slate-800 dark:bg-slate-800">
          <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-tighter text-slate-700 dark:text-slate-200">
            <Clock size={12} />
            {formatDate(request.requested_punch_time)}
          </div>
          <div className="mt-1 font-mono text-sm font-bold text-primary-600 dark:text-primary-400">
            {formatTime(request.requested_punch_time)}
          </div>
        </div>

        <Typography className="text-xs font-bold uppercase tracking-tight leading-relaxed text-slate-500 opacity-70 italic line-clamp-2 dark:text-slate-400" variant="p">
          {request.reason ?? t("common.no_notes")}
        </Typography>

        {/* Attachments */}
        {!!request.attachment_file_ids && request.attachment_file_ids.length > 0 && (
          <AttachmentList fileIds={request.attachment_file_ids} onPreviewAttachment={onPreviewAttachment}  />
        )}
      </div>

      {/* Actions */}
      {!!canApprove && request.status === "pending" && activeTab === "adjustments" && (
        <div className="mt-4 flex justify-end gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
          <Button
            className="p-3 bg-emerald-600 text-white hover:bg-emerald-700 rounded-xl shadow-lg shadow-emerald-100 transition-all active:scale-90"
            onClick={() => { onApprove(request.id, "approved"); }}
            title={t("common.approve")}
            type="button"
            variant="ghost"
          >
            <Check size={18} />
          </Button>
          <Button
            className="p-3 bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 rounded-xl shadow-md border border-rose-100 dark:border-rose-800 transition-all active:scale-90"
            onClick={() => { onApprove(request.id, "rejected"); }}
            title={t("common.reject")}
            type="button"
            variant="ghost"
          >
            <X size={18} />
          </Button>
        </div>
      )}
    </div>
  );
}

interface AdjustmentsTableProps {
  activeTab: string;
  canApprove?: boolean;
  formatDate: (date: string) => string;
  formatTime: (date: string) => string;
  getEmployeeID: (id: string) => string;
  getEmployeeName: (id: string) => string;
  hasNextPage?: boolean;
  isFetchingNextPage?: boolean;
  isLoading: boolean;
  loadMoreRef?: React.Ref<HTMLTableRowElement>;
  onApprove: (id: string, status: string) => void;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  requests: AttendanceAdjustmentRequest[];
  setStatusFilter: (filter: AttendanceStatusFilter) => void;
  statusFilter: AttendanceStatusFilter;
}

export function AdjustmentsTable(props: AdjustmentsTableProps) {
  const {
    activeTab,
    statusFilter,
    setStatusFilter,
    formatDate,
    formatTime,
    getEmployeeID,
    getEmployeeName,
    isLoading,
    onApprove,
    requests,
    canApprove,
    onPreviewAttachment,
    hasNextPage,
    isFetchingNextPage,
    loadMoreRef,
  } = props;

  return (
    <ResponsiveTable<AttendanceAdjustmentRequest>
      data={requests}
      emptyState={{
        title: "No adjustment requests",
        icon: <Clock className="h-10 w-10" />,
      }}
      getCardKey={(request) => request.id}
      isLoading={isLoading}
      renderCard={(request) => (
        <AdjustmentMobileCard
          activeTab={activeTab}
          canApprove={canApprove}
          formatDate={formatDate}
          formatTime={formatTime}
          getEmployeeID={getEmployeeID}
          getEmployeeName={getEmployeeName}
          onApprove={onApprove}
          onPreviewAttachment={onPreviewAttachment}
          request={request}
        />
      )}
      renderDesktopTable={() => (
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl sm:rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <AdjustmentsTableHeader setStatusFilter={setStatusFilter} statusFilter={statusFilter} />
              <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                {isLoading ? (
                  <AdjustmentsTableLoading colSpan={5} />
                ) : requests.length === 0 ? (
                  <AdjustmentsTableEmpty colSpan={5} />
                ) : (
                  <>
                    {requests.map((request) => (
                      <AdjustmentsTableRow
                        activeTab={activeTab}
                        canApprove={canApprove}
                        formatDate={formatDate}
                        formatTime={formatTime}
                        getEmployeeID={getEmployeeID}
                        getEmployeeName={getEmployeeName}
                        key={request.id}
                        onApprove={onApprove}
                        onPreviewAttachment={onPreviewAttachment}
                        request={request}
                      />
                    ))}
                    {!!hasNextPage && !!loadMoreRef && (
                      <InfiniteScrollSentinel
                        colSpan={5}
                        isFetchingNextPage={!!isFetchingNextPage}
                        sentinelRef={loadMoreRef}
                      />
                    )}
                  </>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    />
  );
}
