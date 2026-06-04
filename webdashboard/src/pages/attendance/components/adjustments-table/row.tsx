import { Check, User as UserIcon, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { AttendanceAdjustmentRequest } from "@/lib/api/types";

interface AdjustmentsTableRowProps {
  activeTab: string;
  canApprove?: boolean;
  formatDate: (date: string) => string;
  formatTime: (date: string) => string;
  getEmployeeID: (id: string) => string;
  getEmployeeName: (id: string) => string;
  onApprove: (id: string, status: string) => void;
  request: AttendanceAdjustmentRequest;
}

export function AdjustmentsTableRow({
  activeTab,
  formatDate,
  formatTime,
  getEmployeeID,
  getEmployeeName,
  onApprove,
  request,
  canApprove,
}: AdjustmentsTableRowProps) {
  const { t } = useTranslation();

  return (
    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group" key={request.id}>
      <td className="px-10 py-6">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold shadow-inner border border-primary-100 dark:border-primary-800/50 overflow-hidden">
            <UserIcon size={20} />
          </div>
          <div>
            <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1">
              {getEmployeeName(request.employee_id)}
            </div>
            <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold uppercase opacity-70">
              ID: {getEmployeeID(request.employee_id)}
            </div>
          </div>
        </div>
      </td>
      <td className="px-8 py-6 text-center">
        <div className="inline-flex flex-col items-center bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-inner">
          <div className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tighter">
            {formatDate(request.requested_punch_in_time)}
          </div>
          <div className="text-[10px] font-mono font-bold text-primary-600 dark:text-primary-400 mt-1">
            {formatTime(request.requested_punch_in_time)} —{" "}
            {request.requested_punch_out_time ? formatTime(request.requested_punch_out_time) : "???"}
          </div>
        </div>
      </td>
      <td className="px-8 py-6">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tight line-clamp-2 italic opacity-70 leading-relaxed" variant="p">
          {request.reason ?? t("common.no_notes")}
        </Typography>
      </td>
      <td className="px-8 py-6 text-center">
        <StatusBadge
          variant={
            request.status === "pending" ? "warning" : request.status === "approved" ? "success" : "danger"
          }
        >
          {t(`common.status_${request.status}`).toUpperCase()}
        </StatusBadge>
      </td>
      <td className="px-10 py-6 text-right">
        {!!canApprove && request.status === "pending" && activeTab === "pending" && (
          <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">
            <Button className="p-3 bg-emerald-600 text-white hover:bg-emerald-700 rounded-xl shadow-lg shadow-emerald-100 transition-all active:scale-90"
              onClick={() => { onApprove(request.id, "approved"); }}
              title={t("common.approve")}
              type="button"
              variant="ghost"
            >
              <Check size={18} />
            </Button>
            <Button className="p-3 bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 rounded-xl shadow-md border border-rose-100 dark:border-rose-800 transition-all active:scale-90"
              onClick={() => { onApprove(request.id, "rejected"); }}
              title={t("common.reject")}
              type="button"
              variant="ghost"
            >
              <X size={18} />
            </Button>
          </div>
        )}
      </td>
    </tr>
  );
}
