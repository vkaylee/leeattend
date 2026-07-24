import * as Dialog from "@radix-ui/react-dialog";
import { X, FileText } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { DailyAttendanceReport } from "@/lib/api";

import Button from "@/components/ui/button";
import { useConfirmClaimsForm } from "@/hooks/use-confirm-claims-form";

import { ClaimsFormView } from "./claims-form-view";
import { DiscrepancyWarningsView } from "./discrepancy-warnings-view";

// ── Types ──────────────────────────────────────────────────────────────

type ReportRow = DailyAttendanceReport & { employee_id_number?: string };

export interface ClaimItem {
  claimed_minutes: number;
  computed_minutes: number;
  date: string;
  employee_id: string;
  employee_id_number: string;
  employee_name: string;
  employment_id: string;
  reason: string;
}

interface Props {
  initialReasons?: Record<string, string>;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  open: boolean;
  selectedRows: ReportRow[];
}

// ── Component ──────────────────────────────────────────────────────────

export function ConfirmClaimsDialog({
  open,
  onOpenChange,
  selectedRows,
  initialReasons = {},
  onSuccess,
}: Props) {
  const { t } = useTranslation(["attendance", "common"]);

  const {
    claims,
    photoUrl,
    fileName,
    showConfirmDiscrepancies,
    setShowConfirmDiscrepancies,
    error,
    isSubmitting,
    handleMinutesChange,
    handleReasonChange,
    handleFileChange,
    handleSubmit,
    setPhotoUrl,
    setFileName,
  } = useConfirmClaimsForm({
    open,
    onOpenChange,
    selectedRows,
    initialReasons,
    onSuccess,
  });

  const discrepancies = claims.filter((c) => c.claimed_minutes !== c.computed_minutes);

  return (
    <Dialog.Root onOpenChange={onOpenChange} open={open}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl -translate-x-1/2 -translate-y-1/2 border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 p-6 shadow-xl rounded-[2rem] overflow-hidden">
          <Dialog.Title className="text-xl font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight flex items-center gap-2">
            <FileText className="w-5 h-5" />
            {showConfirmDiscrepancies
              ? t("ot_approvals.delegated_claim.warning.title")
              : t("ot_approvals.delegated_claim.confirm_title")}
          </Dialog.Title>
          <Dialog.Description className="text-sm text-slate-500 dark:text-slate-400 mt-1">
            {showConfirmDiscrepancies
              ? t("ot_approvals.delegated_claim.warning.description")
              : t("ot_approvals.delegated_claim.confirm_description")}
          </Dialog.Description>

          <Button
            aria-label={t("common.close")}
            className="absolute right-4 top-4 h-auto w-auto p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
            icon={<X className="h-4 w-4" />}
            onClick={() => { onOpenChange(false); }}
            size="icon"
            variant="ghost"
          />

          {!showConfirmDiscrepancies ? (
            <ClaimsFormView
              claims={claims}
              error={error}
              fileName={fileName}
              handleFileChange={handleFileChange}
              handleMinutesChange={handleMinutesChange}
              handleReasonChange={handleReasonChange}
              isSubmitting={isSubmitting}
              onCancel={() => { onOpenChange(false); }}
              onSubmit={(e) => { e.preventDefault(); void handleSubmit(); }}
              photoUrl={photoUrl}
              setFileName={setFileName}
              setPhotoUrl={setPhotoUrl}
              t={t}
            />
          ) : (
            <DiscrepancyWarningsView
              discrepancies={discrepancies}
              error={error}
              isSubmitting={isSubmitting}
              onConfirm={() => void handleSubmit()}
              onGoBack={() => { setShowConfirmDiscrepancies(false); }}
              t={t}
            />
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
