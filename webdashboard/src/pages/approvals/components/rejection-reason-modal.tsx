import { AlertTriangle, Loader2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";

interface RejectionReasonModalProps {
  isOpen: boolean;
  itemCount: number;
  isProcessing: boolean;
  onClose: () => void;
  onConfirm: (reason: string) => void;
}

export function RejectionReasonModal({
  isOpen,
  itemCount,
  isProcessing,
  onClose,
  onConfirm,
}: RejectionReasonModalProps) {
  const { t } = useTranslation();
  const [reason, setReason] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = reason.trim();
    if (!trimmed) {
      setError(
        t(
          "approvals.rejection.reason_required",
          "Vui lòng nhập lý do từ chối yêu cầu"
        )
      );
      return;
    }
    setError(null);
    onConfirm(trimmed);
  };

  const handleClose = () => {
    if (isProcessing) return;
    setReason("");
    setError(null);
    onClose();
  };

  return (
    <Modal
      icon={<AlertTriangle className="h-5 w-5 text-rose-500" />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={handleClose}
      subtitle={t(
        "approvals.rejection.subtitle",
        "Lý do này sẽ được gửi thông báo đến nhân sự liên quan"
      )}
      testId="rejection-reason-modal"
      title={t("approvals.rejection.title", "Xác nhận từ chối {{count}} yêu cầu", {
        count: itemCount,
      })}
    >
      <form className="space-y-4 pt-2" onSubmit={handleSubmit}>
        <div className="space-y-1.5">
          <label
            className="block text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300"
            htmlFor="rejection-reason-input"
          >
            {t("approvals.rejection.reason_label", "Lý do từ chối *")}
          </label>
          <textarea
            aria-describedby={error ? "rejection-reason-error" : undefined}
            aria-invalid={!!error}
            className="w-full min-h-[110px] rounded-2xl border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-900 p-3.5 text-sm text-slate-900 dark:text-slate-100 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-rose-500 transition-all resize-none"
            data-testid="rejection-reason-textarea"
            disabled={isProcessing}
            id="rejection-reason-input"
            onChange={(e) => {
              setReason(e.target.value);
              if (error) setError(null);
            }}
            placeholder={t(
              "approvals.rejection.placeholder",
              "Nhập lý do cụ thể (ví dụ: Thiếu nhân sự trực ca, thời gian tăng ca chưa được duyệt trước)..."
            )}
            value={reason}
          />
          {error && (
            <p
              className="text-xs font-medium text-rose-600 dark:text-rose-400 animate-in fade-in"
              id="rejection-reason-error"
            >
              {error}
            </p>
          )}
        </div>

        <div className="flex items-center justify-end gap-3 pt-2">
          <Button
            data-testid="rejection-cancel-btn"
            disabled={isProcessing}
            onClick={handleClose}
            size="md"
            type="button"
            variant="ghost"
          >
            {t("common.cancel", "Hủy bỏ")}
          </Button>
          <Button
            className="bg-rose-600 hover:bg-rose-500 text-white font-semibold"
            data-testid="rejection-confirm-btn"
            disabled={isProcessing}
            size="md"
            type="submit"
            variant="primary"
          >
            {isProcessing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t("approvals.rejection.confirm_btn", "Xác nhận từ chối")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
