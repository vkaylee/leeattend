import { Check, Loader2, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

interface ApprovalBatchToolbarProps {
  selectedCount: number;
  isProcessing: boolean;
  onApproveAll: () => void;
  onRejectAll: () => void;
  onClearSelection: () => void;
}

export function ApprovalBatchToolbar({
  selectedCount,
  isProcessing,
  onApproveAll,
  onRejectAll,
  onClearSelection,
}: ApprovalBatchToolbarProps) {
  const { t } = useTranslation();

  if (selectedCount === 0) return null;

  return (
    <div
      aria-label={t("approvals.batch.toolbar_label", "Thanh thao tác hàng loạt")}
      className="fixed bottom-24 sm:bottom-28 left-1/2 -translate-x-1/2 z-40 flex items-center gap-3 px-5 py-3 rounded-2xl bg-slate-950/90 dark:bg-slate-900/95 text-white backdrop-blur-2xl border border-slate-800 shadow-2xl animate-in slide-in-from-bottom-5 duration-300"
      data-testid="approval-batch-toolbar"
      role="toolbar"
    >
      <div className="flex items-center gap-2 pr-2 border-r border-slate-800">
        <span className="flex h-6 w-6 items-center justify-center rounded-full bg-primary-600 text-xs font-bold text-white">
          {selectedCount}
        </span>
        <span className="text-xs font-medium text-slate-300">
          {t("approvals.batch.selected_count", "yêu cầu đã chọn")}
        </span>
      </div>

      <Button
        className="h-9 px-4 rounded-xl font-semibold bg-emerald-600 hover:bg-emerald-500 text-white shadow-md transition-all active:scale-95"
        data-testid="batch-approve-btn"
        disabled={isProcessing}
        onClick={onApproveAll}
        size="sm"
        variant="primary"
      >
        {isProcessing ? (
          <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
        ) : (
          <Check className="mr-1.5 h-4 w-4 stroke-[2.5]" />
        )}
        {t("approvals.batch.approve_btn", "Duyệt tất cả")}
      </Button>

      <Button
        className="h-9 px-4 rounded-xl font-semibold bg-rose-600/90 hover:bg-rose-500 text-white border-0 shadow-md transition-all active:scale-95"
        data-testid="batch-reject-btn"
        disabled={isProcessing}
        onClick={onRejectAll}
        size="sm"
        variant="outline"
      >
        <X className="mr-1.5 h-4 w-4 stroke-[2.5]" />
        {t("approvals.batch.reject_btn", "Từ chối")}
      </Button>

      <Button
        aria-label={t("common.cancel", "Hủy chọn")}
        className="h-8 w-8 p-0 rounded-full text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        data-testid="batch-clear-btn"
        disabled={isProcessing}
        onClick={onClearSelection}
        size="sm"
        variant="ghost"
      >
        <X className="h-4 w-4" />
      </Button>
    </div>
  );
}
