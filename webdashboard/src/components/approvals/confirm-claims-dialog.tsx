import * as Dialog from "@radix-ui/react-dialog";
import { X, Upload, FileText, AlertTriangle, ArrowRight } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { DailyAttendanceReport } from "@/lib/api";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useClaimOt, useEnterPaperOt } from "@/hooks/use-attendance";
import { cn } from "@/lib/utils";

type ReportRow = DailyAttendanceReport & { employee_id_number?: string };

interface Props {
  initialReasons?: Record<string, string>;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  open: boolean;
  selectedRows: ReportRow[];
}

interface ClaimItem {
  claimed_minutes: number;
  computed_minutes: number;
  date: string;
  employee_id: string;
  employee_id_number: string;
  employee_name: string;
  reason: string;
}

export function ConfirmClaimsDialog({
  open,
  onOpenChange,
  selectedRows,
  initialReasons = {},
  onSuccess,
}: Props) {
  const { t } = useTranslation(["common", "attendance"]);
  const toast = useToast();

  const { enterPaperOt, isSubmitting: isSubmittingPaper } = useEnterPaperOt();
  const { claimOt, isSubmitting: isSubmittingClaim } = useClaimOt();

  const isSubmitting = isSubmittingPaper || isSubmittingClaim;

  // Form states
  const [claims, setClaims] = useState<ClaimItem[]>([]);
  const [photoUrl, setPhotoUrl] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [showConfirmDiscrepancies, setShowConfirmDiscrepancies] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [prevOpen, setPrevOpen] = useState(open);
  if (open !== prevOpen) {
    setPrevOpen(open);
    if (open) {
      setClaims(
        selectedRows.map((r) => {
          const key = `${r.employee_id}-${r.date}`;
          const computed_minutes = Math.round((r.computed_ot_hours) * 60);
          return {
            employee_id: r.employee_id,
            date: r.date,
            employee_name: r.employee_name,
            employee_id_number: r.employee_id_number ?? "",
            computed_minutes,
            claimed_minutes: computed_minutes,
            reason: initialReasons[key] ?? t("ot_approvals.delegated_claim.default_reason", { defaultValue: "Delegated OT claim" }),
          };
        })
      );
      setPhotoUrl(null);
      setFileName(null);
      setShowConfirmDiscrepancies(false);
      setError(null);
    }
  }

  const handleMinutesChange = (index: number, val: number) => {
    setClaims((prev) =>
      prev.map((c, i) => (i === index ? { ...c, claimed_minutes: val } : c))
    );
  };

  const handleReasonChange = (index: number, val: string) => {
    setClaims((prev) =>
      prev.map((c, i) => (i === index ? { ...c, reason: val } : c))
    );
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setFileName(file.name);
      setPhotoUrl(`/uploads/mock_${file.name}`);
    }
  };

  const validateForm = () => {
    const hasInvalidMinutes = claims.some((c) => c.claimed_minutes <= 0);
    if (hasInvalidMinutes) {
      setError(t("approvals.paper_ot.error_minutes", { defaultValue: "Minutes must be > 0" }));
      return false;
    }

    const hasEmptyReason = claims.some((c) => !c.reason.trim());
    if (hasEmptyReason) {
      setError(t("approvals.paper_ot.error_reason", { defaultValue: "Reason is required" }));
      return false;
    }

    return true;
  };

  const handleSubmit = async (e?: React.SyntheticEvent) => {
    e?.preventDefault();
    setError(null);

    if (!validateForm()) return;

    // Check for discrepancies
    const discrepanciesList = claims.filter(
      (c) => c.claimed_minutes !== c.computed_minutes
    );

    if (discrepanciesList.length > 0 && !showConfirmDiscrepancies) {
      setShowConfirmDiscrepancies(true);
      return;
    }

    try {
      const promises = claims.map((c) => {
        if (photoUrl) {
          return enterPaperOt({
            employeeId: c.employee_id,
            date: c.date,
            claimed_minutes: c.claimed_minutes,
            reason: c.reason,
            paper_form_photo_url: photoUrl,
          });
        } else {
          return claimOt({
            employeeId: c.employee_id,
            date: c.date,
            claimed_minutes: c.claimed_minutes,
            reason: c.reason,
          });
        }
      });

      await Promise.all(promises);

      toast.success(t("ot_approvals.delegated_claim.messages.success", { defaultValue: "Overtime claimed successfully" }));
      onOpenChange(false);
      onSuccess?.();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Failed to claim overtime";
      setError(message);
    }
  };

  const discrepancies = claims.filter((c) => c.claimed_minutes !== c.computed_minutes);

  return (
    <Dialog.Root onOpenChange={onOpenChange} open={open}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl -translate-x-1/2 -translate-y-1/2 border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 p-6 shadow-xl rounded-[2rem] overflow-hidden">
          <Dialog.Title className="text-xl font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight flex items-center gap-2">
            <FileText className="w-5 h-5" />
            {showConfirmDiscrepancies
              ? t("ot_approvals.delegated_claim.warning.title", { defaultValue: "Verify Discrepancies" })
              : t("ot_approvals.delegated_claim.confirm_title", { defaultValue: "Confirm Delegated Claims" })}
          </Dialog.Title>
          <Dialog.Description className="text-sm text-slate-500 dark:text-slate-400 mt-1">
            {showConfirmDiscrepancies
              ? t("ot_approvals.delegated_claim.warning.description", { defaultValue: "Some claims differ from system-detected OT. Please verify before proceeding." })
              : t("ot_approvals.delegated_claim.confirm_description", { defaultValue: "Review and edit overtime claims on behalf of employees." })}
          </Dialog.Description>

          <Button
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

interface ClaimsFormViewProps {
  claims: ClaimItem[];
  error: string | null;
  fileName: string | null;
  handleFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  handleMinutesChange: (index: number, val: number) => void;
  handleReasonChange: (index: number, val: string) => void;
  isSubmitting: boolean;
  onCancel: () => void;
  onSubmit: (e: React.SyntheticEvent) => void;
  photoUrl: string | null;
  setFileName: (val: string | null) => void;
  setPhotoUrl: (val: string | null) => void;
  t: (key: string, defaultValue?: { defaultValue: string }) => string;
}

function ClaimsFormView({
  claims,
  error,
  fileName,
  handleFileChange,
  handleMinutesChange,
  handleReasonChange,
  isSubmitting,
  onCancel,
  onSubmit,
  photoUrl,
  setFileName,
  setPhotoUrl,
  t,
}: ClaimsFormViewProps) {
  return (
    <form className="mt-6 space-y-6" onSubmit={onSubmit}>
      {/* Claims Table */}
      <div className="border border-slate-100 dark:border-slate-800 rounded-2xl overflow-hidden">
        <div className="max-h-[250px] overflow-y-auto custom-scrollbar">
          <table className="w-full text-left border-collapse">
            <thead className="sticky top-0 bg-slate-50 dark:bg-slate-800/80 backdrop-blur border-b border-slate-100 dark:border-slate-800 z-10">
              <tr>
                <th className="p-3 pl-4 text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                  {t("ot_approvals.delegated_claim.table.employee", { defaultValue: "Employee" })}
                </th>
                <th className="p-3 text-[10px] font-bold text-slate-500 uppercase tracking-wider w-[140px]">
                  {t("ot_approvals.delegated_claim.table.computed_ot", { defaultValue: "System OT" })}
                </th>
                <th className="p-3 text-[10px] font-bold text-slate-500 uppercase tracking-wider w-[120px]">
                  {t("approvals.paper_ot.minutes", { defaultValue: "Claimed Mins" })}
                </th>
                <th className="p-3 pr-4 text-[10px] font-bold text-slate-500 uppercase tracking-wider w-[200px]">
                  {t("common.reason", { defaultValue: "Reason" })}
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
              {claims.map((claim, idx) => (
                <tr className="hover:bg-slate-50/50 dark:hover:bg-slate-800/20 transition-colors" key={`${claim.employee_id}-${claim.date}`}>
                  <td className="p-3 pl-4">
                    <div className="flex flex-col">
                      <span className="font-bold text-xs text-slate-800 dark:text-slate-200">
                        {claim.employee_name}
                      </span>
                      <span className="text-[9px] font-mono text-slate-400 dark:text-slate-500">
                        {claim.employee_id_number} • {claim.date}
                      </span>
                    </div>
                  </td>
                  <td className="p-3 text-xs text-slate-600 dark:text-slate-400">
                    {claim.computed_minutes} min ({(claim.computed_minutes / 60).toFixed(1)} hrs)
                  </td>
                  <td className="p-3">
                    <div className="flex items-center gap-1.5">
                      <input
                        className="w-[70px] px-2 py-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-xs font-mono text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-primary-500"
                        min={1}
                        onChange={(e) => { handleMinutesChange(idx, parseInt(e.target.value) || 0); }}
                        type="number"
                        value={claim.claimed_minutes}
                      />
                      <span className="text-[10px] text-slate-400 font-mono whitespace-nowrap">
                        {(claim.claimed_minutes / 60).toFixed(1)}h
                      </span>
                    </div>
                  </td>
                  <td className="p-3 pr-4">
                    <input
                      className="w-full px-2 py-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-xs text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-primary-500"
                      onChange={(e) => { handleReasonChange(idx, e.target.value); }}
                      placeholder={t("ot_approvals.delegated_claim.table.reason", { defaultValue: "Enter reason..." })}
                      type="text"
                      value={claim.reason}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Photo Evidence Uploader */}
      <div className="space-y-2">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] flex items-center justify-between">
          <span>{t("approvals.paper_ot.photo", { defaultValue: "Paper Form Photo (Optional)" })}</span>
          {!!photoUrl && (
            <span className="text-[9px] text-amber-600 dark:text-amber-400 font-normal lowercase tracking-normal">
              *{t("ot_approvals.delegated_claim.paper_warning", { defaultValue: "requires paper upload permission" })}
            </span>
          )}
        </label>
        {photoUrl ? (
          <div className="border border-emerald-100 dark:border-emerald-800/50 bg-emerald-50/50 dark:bg-emerald-950/20 rounded-xl p-4 text-center relative group">
            <FileText className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
            <Typography className="text-xs font-semibold text-emerald-800 dark:text-emerald-300" variant="p">
              {fileName ?? "paper_form_evidence.png"}
            </Typography>
            <Button
              className="absolute top-2 right-2 p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full h-auto w-auto shadow-none"
              onClick={() => {
                setPhotoUrl(null);
                setFileName(null);
              }}
              type="button"
              variant="ghost"
            >
              <X size={14} />
            </Button>
          </div>
        ) : (
          <div className="relative border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-xl p-5 text-center hover:border-primary-400 dark:hover:border-primary-500 transition-colors group cursor-pointer">
            <input
              accept="image/*"
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
              onChange={handleFileChange}
              type="file"
            />
            <Upload className="w-6 h-6 text-slate-400 dark:text-slate-500 mx-auto mb-2 group-hover:text-primary-500 transition-colors" />
            <Typography className="text-xs font-semibold text-slate-600 dark:text-slate-300" variant="p">
              {t("approvals.paper_ot.upload_prompt", { defaultValue: "Click or drag photo to upload" })}
            </Typography>
          </div>
        )}
      </div>

      {!!error && (
        <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3">
          <Typography className="text-xs font-semibold text-rose-600 dark:text-rose-300" variant="p">
            {error}
          </Typography>
        </div>
      )}

      {/* Form Actions */}
      <div className="flex justify-end gap-3 pt-2">
        <Button
          className="h-auto px-4 py-2 rounded-xl text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
          onClick={onCancel}
          type="button"
          variant="ghost"
        >
          {t("common.cancel", { defaultValue: "Cancel" })}
        </Button>
        <Button
          className="h-auto px-6 py-2 rounded-xl text-xs font-bold uppercase tracking-wider shadow-lg"
          disabled={isSubmitting}
          loading={isSubmitting}
          type="submit"
          variant="primary"
        >
          {t("common.submit", { defaultValue: "Submit" })}
        </Button>
      </div>
    </form>
  );
}

interface DiscrepancyWarningsViewProps {
  discrepancies: ClaimItem[];
  error: string | null;
  isSubmitting: boolean;
  onConfirm: () => void;
  onGoBack: () => void;
  t: (key: string, defaultValue?: { defaultValue: string }) => string;
}

function DiscrepancyWarningsView({
  discrepancies,
  error,
  isSubmitting,
  onGoBack,
  onConfirm,
  t,
}: DiscrepancyWarningsViewProps) {
  return (
    <div className="mt-6 space-y-6">
      <div className="p-4 bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900 rounded-2xl flex items-start gap-3">
        <AlertTriangle className="text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" size={20} />
        <div className="space-y-1">
          <Typography className="text-xs font-bold text-amber-800 dark:text-amber-300 uppercase tracking-wider" variant="p">
            {t("ot_approvals.delegated_claim.warning.title", { defaultValue: "Time Discrepancies Detected" })}
          </Typography>
          <Typography className="text-xs text-amber-700 dark:text-amber-400" variant="p">
            {t("ot_approvals.delegated_claim.warning.details", { defaultValue: "The following employees have claim times that differ from system computed OT. Please confirm." })}
          </Typography>
        </div>
      </div>

      {/* Discrepancy List */}
      <div className="border border-slate-100 dark:border-slate-800 rounded-2xl max-h-[220px] overflow-y-auto custom-scrollbar divide-y divide-slate-100 dark:divide-slate-800">
        {discrepancies.map((c) => {
          const diff = c.claimed_minutes - c.computed_minutes;
          const diffSign = diff > 0 ? `+${diff}` : `${diff}`;
          return (
            <div className="p-4 flex items-center justify-between text-xs hover:bg-slate-50/50 dark:hover:bg-slate-800/10 transition-colors" key={`${c.employee_id}-${c.date}`}>
              <div>
                <Typography className="font-bold text-slate-800 dark:text-slate-200" variant="p">
                  {c.employee_name}
                </Typography>
                <Typography className="text-[10px] text-slate-400" variant="p">
                  {c.employee_id_number} • {c.date}
                </Typography>
              </div>
              <div className="flex items-center gap-3 font-mono">
                <span className="text-slate-500">
                  {c.computed_minutes}m
                </span>
                <ArrowRight className="text-slate-400" size={14} />
                <span className={cn(
                  "font-bold",
                  diff > 0 ? "text-emerald-600 dark:text-emerald-400" : "text-rose-600 dark:text-rose-400"
                )}>
                  {c.claimed_minutes}m ({diffSign}m)
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {!!error && (
        <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3">
          <Typography className="text-xs font-semibold text-rose-600 dark:text-rose-300" variant="p">
            {error}
          </Typography>
        </div>
      )}

      {/* Discrepancy actions */}
      <div className="flex justify-end gap-3 pt-2">
        <Button
          className="h-auto px-4 py-2 rounded-xl text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
          disabled={isSubmitting}
          onClick={onGoBack}
          type="button"
          variant="ghost"
        >
          {t("ot_approvals.delegated_claim.warning.btn_edit", { defaultValue: "Go Back & Edit" })}
        </Button>
        <Button
          className="h-auto px-6 py-2 rounded-xl text-xs font-bold uppercase tracking-wider shadow-lg bg-amber-600 hover:bg-amber-700 text-white border-amber-600"
          disabled={isSubmitting}
          loading={isSubmitting}
          onClick={onConfirm}
          type="button"
          variant="primary"
        >
          {t("ot_approvals.delegated_claim.warning.btn_confirm", { defaultValue: "Confirm & Submit" })}
        </Button>
      </div>
    </div>
  );
}
