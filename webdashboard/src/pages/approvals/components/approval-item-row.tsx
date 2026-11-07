import {
  ArrowLeftRight,
  Calendar,
  Check,
  Clock,
  FileEdit,
  X,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import {
  ApprovalType,
  UnifiedApprovalItem,
} from "../types/unified-approvals";

interface ApprovalItemRowProps {
  item: UnifiedApprovalItem;
  isSelected: boolean;
  onToggleSelect: (id: string) => void;
  onClick: (item: UnifiedApprovalItem) => void;
  onQuickApprove: (item: UnifiedApprovalItem) => void;
  onQuickReject: (item: UnifiedApprovalItem) => void;
  isProcessing?: boolean;
}

const typeIconMap: Record<ApprovalType, React.ReactNode> = {
  leave: <Calendar className="h-4 w-4" />,
  overtime: <Clock className="h-4 w-4" />,
  adjustment: <FileEdit className="h-4 w-4" />,
  shift_swap: <ArrowLeftRight className="h-4 w-4" />,
};

const typeColorStyles: Record<
  ApprovalType,
  { badge: string; iconBg: string; text: string }
> = {
  leave: {
    badge:
      "bg-sky-50 text-sky-700 border-sky-200/80 dark:bg-sky-950/60 dark:text-sky-300 dark:border-sky-800/60",
    iconBg: "bg-sky-100 text-sky-600 dark:bg-sky-900/60 dark:text-sky-300",
    text: "text-sky-700 dark:text-sky-300",
  },
  overtime: {
    badge:
      "bg-violet-50 text-violet-700 border-violet-200/80 dark:bg-violet-950/60 dark:text-violet-300 dark:border-violet-800/60",
    iconBg:
      "bg-violet-100 text-violet-600 dark:bg-violet-900/60 dark:text-violet-300",
    text: "text-violet-700 dark:text-violet-300",
  },
  adjustment: {
    badge:
      "bg-amber-50 text-amber-700 border-amber-200/80 dark:bg-amber-950/60 dark:text-amber-300 dark:border-amber-800/60",
    iconBg:
      "bg-amber-100 text-amber-600 dark:bg-amber-900/60 dark:text-amber-300",
    text: "text-amber-700 dark:text-amber-300",
  },
  shift_swap: {
    badge:
      "bg-teal-50 text-teal-700 border-teal-200/80 dark:bg-teal-950/60 dark:text-teal-300 dark:border-teal-800/60",
    iconBg: "bg-teal-100 text-teal-600 dark:bg-teal-900/60 dark:text-teal-300",
    text: "text-teal-700 dark:text-teal-300",
  },
};

export function ApprovalItemRow({
  item,
  isSelected,
  onToggleSelect,
  onClick,
  onQuickApprove,
  onQuickReject,
  isProcessing = false,
}: ApprovalItemRowProps) {
  const { t } = useTranslation();
  const { badge, iconBg } = typeColorStyles[item.type];
  const isPending = item.status === "pending";

  const handleRowClick = (e: React.MouseEvent) => {
    // Prevent triggering drawer if user clicked checkbox or quick action buttons
    const target = e.target as HTMLElement;
    if (
      target.closest("button") ||
      target.closest("input[type='checkbox']") ||
      target.closest("label")
    ) {
      return;
    }
    onClick(item);
  };

  return (
    <div
      aria-label={`${item.title} - ${item.requester.name}`}
      className={cn(
        "group relative flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-4 rounded-2xl border transition-all cursor-pointer",
        isSelected
          ? "bg-primary-50/60 dark:bg-primary-950/30 border-primary-400 dark:border-primary-600 shadow-sm"
          : "bg-white dark:bg-slate-900 border-slate-200/80 dark:border-slate-800 hover:border-slate-300 dark:hover:border-slate-700 hover:shadow-md"
      )}
      data-testid={`approval-item-${item.id}`}
      onClick={handleRowClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick(item);
        }
      }}
    >
      {/* Left: Checkbox + Type Icon + Requester & Summary */}
      <div className="flex items-start gap-3.5 min-w-0">
        {/* Checkbox for batch select */}
        {isPending ? (
            <input
              aria-label={t(
                "approvals.select_item",
                "Chọn yêu cầu của {{name}}",
                { name: item.requester.name }
              )}
              checked={isSelected}
              className="h-4 w-4 rounded border-slate-300 text-primary-600 focus:ring-primary-500 cursor-pointer"
              data-testid={`checkbox-${item.id}`}
              disabled={isProcessing}
              onChange={() => onToggleSelect(item.id)}
              type="checkbox"
            />
        ) : (
          <div className="w-4" />
        )}

        {/* Type Icon Squircle */}
        <div
          className={cn(
            "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl",
            iconBg
          )}
        >
          {typeIconMap[item.type]}
        </div>

        {/* Requester & Summary info */}
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="font-bold text-sm text-slate-900 dark:text-slate-100 truncate">
              {item.requester.name}
            </span>
            {item.requester.idNumber && (
              <span className="text-xs font-mono text-slate-400 dark:text-slate-500">
                #{item.requester.idNumber}
              </span>
            )}
            {item.requester.departmentName && (
              <span className="text-xs px-2 py-0.5 rounded-md bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 font-medium truncate max-w-[150px]">
                {item.requester.departmentName}
              </span>
            )}
            <span
              className={cn(
                "text-[11px] font-semibold px-2 py-0.5 rounded-full border",
                badge
              )}
            >
              {item.title}
            </span>
          </div>

          <p className="text-xs text-slate-600 dark:text-slate-400 font-medium truncate">
            {item.subtitle}
          </p>

          {item.details.reason && (
            <p className="text-xs text-slate-500 dark:text-slate-500 italic line-clamp-1">
              &quot;{item.details.reason}&quot;
            </p>
          )}
        </div>
      </div>

      {/* Right: Date, Impact Badge & Quick Inline Actions */}
      <div className="flex items-center justify-between sm:justify-end gap-3 shrink-0 pt-2 sm:pt-0 border-t sm:border-t-0 border-slate-100 dark:border-slate-800">
        <div className="text-left sm:text-right space-y-0.5">
          <p className="text-xs font-semibold text-slate-800 dark:text-slate-200">
            {item.targetDate}
          </p>
          {item.impact && item.impact.colleaguesAwayCount > 0 && (
            <span
              className={cn(
                "inline-flex items-center text-[10px] font-bold px-2 py-0.5 rounded-full",
                item.impact.isHighImpact
                  ? "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300"
                  : "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300"
              )}
            >
              {item.impact.colleaguesAwayCount} người nghỉ cùng ngày
            </span>
          )}
        </div>

        {/* Quick inline action buttons (if pending) */}
        {isPending ? (
          <>
            <Button
              aria-label={t("approvals.quick_approve", "Duyệt nhanh")}
              className="h-8 w-8 p-0 rounded-xl bg-emerald-50 text-emerald-600 hover:bg-emerald-600 hover:text-white dark:bg-emerald-950/60 dark:text-emerald-400 dark:hover:bg-emerald-600 border border-emerald-200 dark:border-emerald-800 transition-all active:scale-95"
              data-testid={`quick-approve-${item.id}`}
              disabled={isProcessing}
              onClick={() => onQuickApprove(item)}
              size="sm"
              variant="outline"
            >
              <Check className="h-4 w-4 stroke-[2.5]" />
            </Button>
            <Button
              aria-label={t("approvals.quick_reject", "Từ chối")}
              className="h-8 w-8 p-0 rounded-xl bg-rose-50 text-rose-600 hover:bg-rose-600 hover:text-white dark:bg-rose-950/60 dark:text-rose-400 dark:hover:bg-rose-600 border border-rose-200 dark:border-rose-800 transition-all active:scale-95"
              data-testid={`quick-reject-${item.id}`}
              disabled={isProcessing}
              onClick={() => onQuickReject(item)}
              size="sm"
              variant="outline"
            >
              <X className="h-4 w-4 stroke-[2.5]" />
            </Button>
          </>
        ) : (
          <span
            className={cn(
              "text-xs font-bold px-2.5 py-1 rounded-xl uppercase tracking-wider",
              item.status === "approved"
                ? "bg-emerald-100 dark:bg-emerald-950/80 text-emerald-800 dark:text-emerald-300"
                : "bg-rose-100 dark:bg-rose-950/80 text-rose-800 dark:text-rose-300"
            )}
          >
            {item.status === "approved"
              ? t("common.approved", "Đã duyệt")
              : t("common.rejected", "Đã từ chối")}
          </span>
        )}
      </div>
    </div>
  );
}
