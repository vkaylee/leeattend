import React from "react";

import { AttendanceAdjustmentRequest } from "@/lib/api/types";
import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";

import { AdjustmentsTableEmpty } from "./adjustments-table/empty-state";
import { AdjustmentsTableHeader } from "./adjustments-table/header";
import { AdjustmentsTableLoading } from "./adjustments-table/loading-state";
import { AdjustmentsTableRow } from "./adjustments-table/row";
import type { AttendanceStatusFilter } from "../hooks/use-attendance-adjustments-page";

interface AdjustmentsTableProps {
  activeTab: string;
  statusFilter: AttendanceStatusFilter;
  setStatusFilter: (filter: AttendanceStatusFilter) => void;
  canApprove?: boolean;
  formatDate: (date: string) => string;
  formatTime: (date: string) => string;
  getEmployeeID: (id: string) => string;
  getEmployeeName: (id: string) => string;
  isLoading: boolean;
  onApprove: (id: string, status: string) => void;
  requests: AttendanceAdjustmentRequest[];
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  hasNextPage?: boolean;
  isFetchingNextPage?: boolean;
  loadMoreRef?: React.Ref<HTMLTableRowElement>;
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
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <AdjustmentsTableHeader statusFilter={statusFilter} setStatusFilter={setStatusFilter} />
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
                    request={request}
                    onPreviewAttachment={onPreviewAttachment}
                  />
                ))}
                {hasNextPage && loadMoreRef && (
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
  );
}
