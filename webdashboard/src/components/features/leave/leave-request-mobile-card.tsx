import type { TFunction } from "i18next";

import { CheckCircle2, MessageSquare, Moon, Paperclip, Sun, Timer, XCircle, ShieldCheck  } from "lucide-react";

import type { Employee, LeaveRequest, LeaveType } from "@/lib/api";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useWorkspace } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";

interface LeaveRequestMobileCardProps {
  emp?: Employee;
  formatDate: (d: string) => string;
  leaveTypes: LeaveType[];
  onApprove: (id: string, status: string) => void;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  request: LeaveRequest;
  t: TFunction;
}

// eslint-disable-next-line complexity -- mobile card with 3 duration-type branches + 4 status-style branches; presentation-only
export function LeaveRequestMobileCard({
  request,
  emp,
  leaveTypes,
  formatDate,
  t,
  onApprove,
  onPreviewAttachment,
}: LeaveRequestMobileCardProps) {
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

        <div className="bg-slate-50 dark:bg-slate-800/50 p-3 rounded-xl border border-primary-100 dark:border-primary-800/50 shadow-inner">
          <Typography className="text-[10px] text-primary-400 font-bold leading-relaxed line-clamp-2 italic opacity-80 flex gap-2" variant="p">
            <MessageSquare className="text-slate-300 dark:text-slate-600 mt-0.5 shrink-0" size={12} />
            &quot;{request.reason ?? t("common.no_description")}&quot;
          </Typography>
        </div>

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
