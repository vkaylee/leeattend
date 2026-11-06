import {
  Calendar,
  Check,
  Clock,
  FileText,
  User,
  X,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";

import { UnifiedApprovalItem } from "../types/unified-approvals";

import { TeamScheduleImpactCard } from "./team-schedule-impact-card";

interface ApprovalDetailDrawerProps {
  item: UnifiedApprovalItem | null;
  isOpen: boolean;
  onClose: () => void;
  onApprove: (item: UnifiedApprovalItem) => void;
  onReject: (item: UnifiedApprovalItem) => void;
  isProcessing?: boolean;
}

export function ApprovalDetailDrawer({
  item,
  isOpen,
  onClose,
  onApprove,
  onReject,
  isProcessing = false,
}: ApprovalDetailDrawerProps) {
  const { t } = useTranslation();

  if (!item) return null;

  const isPending = item.status === "pending";

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      side="right"
      subtitle={item.requester.name}
      title={item.title}
    >
      <div
        className="flex flex-col h-full space-y-6 pt-2 pb-6"
        data-testid="approval-detail-drawer"
      >
        {/* Requester Profile Banner */}
        <div className="flex items-center gap-4 p-4 rounded-2xl bg-slate-50 dark:bg-slate-800/50 border border-slate-200/80 dark:border-slate-800">
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-primary-100 dark:bg-primary-950 text-primary-600 dark:text-primary-300 font-bold text-lg">
            {item.requester.name.charAt(0).toUpperCase()}
          </div>
          <div className="space-y-0.5 min-w-0 flex-1">
            <h4 className="font-bold text-base text-slate-900 dark:text-slate-100 truncate">
              {item.requester.name}
            </h4>
            <div className="flex flex-wrap items-center gap-2 text-xs text-slate-500 dark:text-slate-400">
              {item.requester.idNumber && (
                <span className="font-mono">#{item.requester.idNumber}</span>
              )}
              {item.requester.departmentName && (
                <span>• {item.requester.departmentName}</span>
              )}
              {item.requester.jobTitle && (
                <span>• {item.requester.jobTitle}</span>
              )}
            </div>
          </div>
        </div>

        {/* Smart Team Schedule Impact Card */}
        {item.impact && (
          <div className="space-y-2">
            <h5 className="text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
              {t("approvals.drawer.team_context", "Bối cảnh lịch làm việc đội ngũ")}
            </h5>
            <TeamScheduleImpactCard
              impact={item.impact}
              targetDate={item.targetDate}
            />
          </div>
        )}

        {/* Detailed Key-Values */}
        <div className="space-y-3">
          <h5 className="text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
            {t("approvals.drawer.details_title", "Thông tin chi tiết yêu cầu")}
          </h5>

          <div className="divide-y divide-slate-100 dark:divide-slate-800 rounded-2xl border border-slate-200/80 dark:border-slate-800 bg-white dark:bg-slate-900 overflow-hidden text-sm">
            <div className="flex items-center justify-between p-3.5">
              <span className="text-slate-500 dark:text-slate-400 flex items-center gap-2">
                <Calendar className="h-4 w-4" />
                {t("approvals.drawer.target_date", "Thời gian áp dụng")}
              </span>
              <span className="font-semibold text-slate-800 dark:text-slate-200">
                {item.details.dateStr}
              </span>
            </div>

            {item.details.durationLabel && (
              <div className="flex items-center justify-between p-3.5">
                <span className="text-slate-500 dark:text-slate-400 flex items-center gap-2">
                  <Clock className="h-4 w-4" />
                  {t("approvals.drawer.duration", "Thời lượng")}
                </span>
                <span className="font-semibold text-slate-800 dark:text-slate-200">
                  {item.details.durationLabel}
                </span>
              </div>
            )}

            <div className="flex items-center justify-between p-3.5">
              <span className="text-slate-500 dark:text-slate-400 flex items-center gap-2">
                <User className="h-4 w-4" />
                {t("approvals.drawer.status", "Trạng thái")}
              </span>
              <span
                className={`text-xs font-bold px-2.5 py-1 rounded-xl uppercase tracking-wider ${
                  item.status === "approved"
                    ? "bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
                    : item.status === "rejected"
                      ? "bg-rose-100 text-rose-800 dark:bg-rose-950 dark:text-rose-300"
                      : "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300"
                }`}
              >
                {item.status}
              </span>
            </div>

            {item.details.reason && (
              <div className="p-3.5 space-y-1">
                <span className="text-slate-500 dark:text-slate-400 flex items-center gap-2">
                  <FileText className="h-4 w-4" />
                  {t("approvals.drawer.reason", "Lý do")}
                </span>
                <p className="text-slate-800 dark:text-slate-200 pl-6 text-sm whitespace-pre-wrap">
                  {item.details.reason}
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Bottom Actions if Pending */}
        {isPending && (
          <div className="pt-4 flex items-center justify-end gap-3 mt-auto border-t border-slate-200 dark:border-slate-800">
            <Button
              className="bg-rose-600/90 hover:bg-rose-500 text-white font-semibold"
              data-testid="drawer-reject-btn"
              disabled={isProcessing}
              onClick={() => onReject(item)}
              size="md"
              variant="outline"
            >
              <X className="mr-1.5 h-4 w-4 stroke-[2.5]" />
              {t("approvals.batch.reject_btn", "Từ chối")}
            </Button>
            <Button
              className="bg-emerald-600 hover:bg-emerald-500 text-white font-semibold shadow-md"
              data-testid="drawer-approve-btn"
              disabled={isProcessing}
              onClick={() => onApprove(item)}
              size="md"
              variant="primary"
            >
              <Check className="mr-1.5 h-4 w-4 stroke-[2.5]" />
              {t("approvals.batch.approve_btn", "Duyệt yêu cầu")}
            </Button>
          </div>
        )}
      </div>
    </Sheet>
  );
}
