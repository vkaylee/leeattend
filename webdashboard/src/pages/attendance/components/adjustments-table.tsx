import React from "react";

import { AttendanceAdjustmentRequest } from "@/lib/api/types";

import { AdjustmentsTableEmpty } from "./adjustments-table/empty-state";
import { AdjustmentsTableHeader } from "./adjustments-table/header";
import { AdjustmentsTableLoading } from "./adjustments-table/loading-state";
import { AdjustmentsTableRow } from "./adjustments-table/row";

interface AdjustmentsTableProps {
  activeTab: string;
  formatDate: (date: string) => string;
  formatTime: (date: string) => string;
  getEmployeeID: (id: string) => string;
  getEmployeeName: (id: string) => string;
  isLoading: boolean;
  onApprove: (id: string, status: string) => void;
  requests: AttendanceAdjustmentRequest[];
}

export function AdjustmentsTable(props: AdjustmentsTableProps) {
  const {
    activeTab,
    formatDate,
    formatTime,
    getEmployeeID,
    getEmployeeName,
    isLoading,
    onApprove,
    requests,
  } = props;

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <AdjustmentsTableHeader />
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <AdjustmentsTableLoading colSpan={5} />
            ) : requests.length === 0 ? (
              <AdjustmentsTableEmpty colSpan={5} />
            ) : (
              requests.map((request) => (
                <AdjustmentsTableRow
                  activeTab={activeTab}
                  formatDate={formatDate}
                  formatTime={formatTime}
                  getEmployeeID={getEmployeeID}
                  getEmployeeName={getEmployeeName}
                  key={request.id}
                  onApprove={onApprove}
                  request={request}
                />
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
