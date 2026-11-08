import type { TFunction } from "i18next";

import { CheckCircle2, XCircle, ShieldCheck, MessageSquare, Sun, Moon, Timer, Paperclip } from "lucide-react";
import React from "react";

import type { Employee, LeaveType, LeaveRequest } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useWorkspace } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";

type LeaveRequestItem = LeaveRequest;

export interface LeaveRequestRowProps {
  emp?: Employee;
  formatDate: (d: string) => string;
  leaveTypes: LeaveType[];
  onApprove: (id: string, status: string) => void;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  onSelect?: (request: LeaveRequestItem) => void;
  request: LeaveRequestItem;
  t: TFunction;
}
function getDurationBadgeStyle(type: string | null) {
  switch (type) {
    case "first_half":
      return "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-200 dark:border-amber-800";
    case "second_half":
      return "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-200 dark:border-blue-800";
    default:
      return "bg-cyan-50 dark:bg-cyan-950 text-cyan-600 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800";
  }
}

function getDurationContent(request: LeaveRequestItem, t: TFunction) {
  if (request.duration_type === "first_half") {
    return <><Sun size={8} /> {t("leave_requests.duration.first_half")}</>;
  }
  if (request.duration_type === "second_half") {
    return <><Moon size={8} /> {t("leave_requests.duration.second_half")}</>;
  }
  if (request.duration_type === "custom_hours") {
    return <><Timer size={8} /> {request.start_time?.slice(0, 5) ?? ""}–{request.end_time?.slice(0, 5) ?? ""}</>;
  }
  return null;
}

function getStatusBadgeStyle(status: string) {
  switch (status) {
    case "pending":
      return "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800";
    case "approved":
      return "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800";
    case "pre_approved":
      return "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100 dark:border-teal-800";
    default:
      return "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800";
  }
}

function getStatusContent(status: string, t: TFunction) {
  switch (status) {
    case "pending":
      return t("common.status_pending").toUpperCase();
    case "approved":
      return t("common.status_approved").toUpperCase();
    case "pre_approved":
      return (
        <span className="flex items-center gap-1">
          <ShieldCheck size={10} />
          {t("common.status_pre_approved").toUpperCase()}
        </span>
      );
    default:
      return t("common.status_rejected").toUpperCase();
  }
}

function EmployeeCell({ emp }: { emp?: Employee }) {
  return (
    <td className="px-10 py-6">
      <div className="flex items-center gap-5">
        <div className="w-14 h-14 bg-primary-50 dark:bg-primary-950 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-xl shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase group-hover:scale-110 transition-transform">
          {emp?.full_name[0] ?? "?"}
        </div>
        <div>
          <div className="font-bold text-primary-700 dark:text-primary-300 text-sm tracking-tight uppercase leading-none mb-1.5">{emp?.full_name ?? "N/A"}</div>
          <div className="text-[10px] text-primary-400 font-mono font-bold uppercase tracking-widest opacity-70">{emp?.employee_id_number}</div>
        </div>
      </div>
    </td>
  );
}

function ActionsCell({ 
  status, 
  id, 
  onApprove, 
  t 
}: { 
  status: string; 
  id: string; 
  onApprove: (id: string, status: string) => void; 
  t: TFunction; 
}) {
  return (
    <td className="px-10 py-6 text-right">
      {status === "pending" && (
        <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
          <Button
             className="border-primary-100 bg-white text-emerald-400 hover:text-emerald-600 dark:border-primary-800/50 dark:bg-slate-900 dark:text-emerald-300 h-11 w-11 p-3"
            onClick={(e) => { e.stopPropagation(); onApprove(id, "approved"); }}
             size="icon"
             title={t("common.approve")}
             variant="outline"
           >
             <CheckCircle2 size={20} />
           </Button>
          <Button
            className="border-primary-100 bg-white text-rose-400 hover:text-rose-600 dark:border-primary-800/50 dark:bg-slate-900 dark:text-rose-300 h-11 w-11 p-3"
            onClick={(e) => { e.stopPropagation(); onApprove(id, "rejected"); }}
            size="icon"
            title={t("common.reject")}
            variant="outline"
          >
            <XCircle size={20} />
          </Button>
        </div>
      )}
    </td>
  );
}

export function LeaveRequestRow({
  request,
  emp,
  leaveTypes,
  formatDate,
  t,
  onApprove,
   onPreviewAttachment,
  onSelect,
 }: LeaveRequestRowProps) {
  const { workspace } = useWorkspace();
  const wsId = workspace?.id ?? "";
  const leaveTypeName = (leaveTypes.find(tOption => tOption.id === request.leave_type_id)?.name ?? "Unknown").toUpperCase();

  return (
    <tr
      className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group cursor-pointer"
      onClick={() => onSelect?.(request)}
    >
      <EmployeeCell emp={emp} />
      <td className="px-8 py-6">
        <span className="px-3 py-1 bg-slate-100 dark:bg-slate-700 text-primary-700 dark:text-primary-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-primary-200 dark:border-primary-800/50 shadow-sm">
          {leaveTypeName}
        </span>
      </td>
      <td className="px-8 py-6">
        <div className="text-xs font-bold text-primary-700 dark:text-primary-300 uppercase tracking-tight mb-1.5">
          {formatDate(request.start_date)} — {formatDate(request.end_date)}
          {request.duration_type !== "full_day" && (
            <span className={cn(
              "ml-2 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider inline-flex items-center gap-1 border",
              getDurationBadgeStyle(request.duration_type)
            )}>
              {getDurationContent(request, t)}
            </span>
          )}
        </div>
        <div className="bg-slate-50 dark:bg-slate-800/50 p-3 rounded-xl border border-primary-100 dark:border-primary-800/50 shadow-inner group-hover:bg-white dark:hover:bg-slate-900 transition-colors">
          <Typography className="text-[10px] text-primary-400 font-bold leading-relaxed line-clamp-1 italic opacity-80 flex gap-2" variant="p">
            <MessageSquare className="text-slate-300 dark:text-slate-600 mt-0.5 shrink-0" size={12} /> 
            &quot;{request.reason ?? t("common.no_description")}&quot;
          </Typography>
        </div>
        {!!request.attachment_file_ids && request.attachment_file_ids.length > 0 && (
          <div className="mt-2 flex flex-wrap gap-2">
            {request.attachment_file_ids.map((fileId, index) => {
              if (!fileId) return null;
              return (
                <Button
                  className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[10px] font-bold bg-indigo-50 hover:bg-indigo-100 text-indigo-750 dark:bg-indigo-950/30 dark:hover:bg-indigo-900/40 dark:text-indigo-400 border border-indigo-150 dark:border-indigo-900/50 transition-colors cursor-pointer h-auto py-1 shadow-none"
                  key={fileId}
                  onClick={() => onPreviewAttachment?.(`/api/v1/files/${fileId}/download`, `Leave Attachment #${index + 1}`)}
                  variant="ghost"
                >
                  <Paperclip className="w-3 h-3" />
                  <span>Doc #{index + 1}</span>
                </Button>
              );
            })}
          </div>
        )}
      </td>
      <td className="px-8 py-6 text-center">
        <span className={cn(
          "px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm inline-block",
          getStatusBadgeStyle(request.status)
        )}>
          {getStatusContent(request.status, t)}
        </span>
      </td>
      <ActionsCell id={request.id} onApprove={onApprove} status={request.status} t={t} />
    </tr>
  );
}

