import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { X, Upload, FileText, Calendar } from "lucide-react";
import { useTranslation } from "react-i18next";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { useEnterPaperOt } from "@/hooks/use-attendance";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import { format } from "date-fns";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

export function EnterPaperOtDialog({ open, onOpenChange, onSuccess }: Props) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const { enterPaperOt, isSubmitting } = useEnterPaperOt();

  // Form state
  const [employeeId, setEmployeeId] = useState<string>("");
  const [date, setDate] = useState(format(new Date(), "yyyy-MM-dd"));
  const [minutes, setMinutes] = useState(0);
  const [reason, setReason] = useState("");
  const [photoUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const resetForm = () => {
    setEmployeeId("");
    setDate(format(new Date(), "yyyy-MM-dd"));
    setMinutes(0);
    setReason("");
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    // Validation
    if (!employeeId) {
      setError(t("approvals.paper_ot.error_no_employee") || "Select an employee");
      return;
    }
    if (minutes <= 0) {
      setError(t("approvals.paper_ot.error_minutes") || "Minutes must be > 0");
      return;
    }
    if (!reason.trim()) {
      setError(t("approvals.paper_ot.error_reason") || "Reason is required");
      return;
    }

    try {
      await enterPaperOt({
        employeeId,
        date,
        claimed_minutes: minutes,
        reason,
        paper_form_photo_url: photoUrl || undefined,
      });
      resetForm();
      onOpenChange(false);
      onSuccess?.();
    } catch (e: any) {
      setError(e.message || "Failed to submit");
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={(o) => { if (!o) resetForm(); onOpenChange(o); }}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 p-6 shadow-xl rounded-[2rem]">
          <Dialog.Title className="text-xl font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight flex items-center gap-2">
            <FileText className="w-5 h-5" />
            {t("approvals.paper_ot.title") || "Enter Paper OT Form"}
          </Dialog.Title>
          <Dialog.Description className="text-sm text-slate-500 dark:text-slate-400 mt-1">
            {t("approvals.paper_ot.description") || "Enter overtime claim from signed paper form"}
          </Dialog.Description>

          <button onClick={() => onOpenChange(false)} className="absolute right-4 top-4 p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700">
            <X className="h-4 w-4" />
          </button>

          <form onSubmit={handleSubmit} className="mt-6 space-y-5">
            {/* Employee Selector */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                {t("common.employee") || "Employee"}
              </label>
              <EmployeeSearch
                workspaceId={wsId}
                value={employeeId}
                onChange={setEmployeeId}
                placeholder={t("approvals.paper_ot.select_employee") || "Search employee..."}
              />
            </div>

            {/* Date Picker */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                {t("common.date") || "Date"}
              </label>
              <div className="relative">
                <Calendar className="absolute left-3 top-3 h-4 w-4 text-slate-400 dark:text-slate-500" />
                <input
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  max={format(new Date(), "yyyy-MM-dd")}
                  className="w-full pl-10 h-10 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-3 text-sm focus:ring-2 focus:ring-primary-600"
                />
              </div>
            </div>

            {/* Minutes Input */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                {t("approvals.paper_ot.minutes") || "OT Minutes"}
              </label>
              <input
                type="number"
                value={minutes || ""}
                onChange={(e) => setMinutes(parseInt(e.target.value) || 0)}
                min={1}
                className="w-full h-10 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-3 text-sm font-mono focus:ring-2 focus:ring-primary-600"
                placeholder="120"
              />
            </div>

            {/* Reason Textarea */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                {t("common.reason") || "Reason"}
              </label>
              <textarea
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                rows={3}
                className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-3 py-2 text-sm resize-none focus:ring-2 focus:ring-primary-600"
                placeholder={t("approvals.paper_ot.reason_placeholder") || "Paper form #1234 - Production line"}
              />
            </div>

            {/* Photo Upload Placeholder (Optional) */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                {t("approvals.paper_ot.photo") || "Paper Form Photo (Optional)"}
              </label>
              <div className="border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-xl p-4 text-center hover:border-primary-400 transition-colors cursor-not-allowed opacity-50">
                <Upload className="w-8 h-8 text-slate-400 dark:text-slate-500 mx-auto mb-2" />
                <p className="text-xs text-slate-500 dark:text-slate-400">{t("approvals.paper_ot.photo_coming_soon") || "Photo upload coming soon"}</p>
              </div>
            </div>

            {/* Error Message */}
            {error && (
              <div className="text-xs font-bold text-rose-500 uppercase tracking-wide bg-rose-50 dark:bg-rose-950 p-3 rounded-lg">
                {error}
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex justify-end gap-3 pt-2">
              <button
                type="button"
                onClick={() => onOpenChange(false)}
                className="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
              >
                {t("common.cancel") || "Cancel"}
              </button>
              <button
                type="submit"
                disabled={isSubmitting}
                className="px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-primary-700 disabled:opacity-50"
              >
                {isSubmitting ? (t("common.saving") || "Saving...") : (t("common.submit") || "Submit")}
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
