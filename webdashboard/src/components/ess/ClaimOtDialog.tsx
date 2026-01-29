import { useState, useEffect } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { X, Clock, AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { AttendanceDailySummary } from "@/lib/api/types";
import { useSubmitOtClaim } from "@/hooks/use-ess";

interface ClaimOtDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  summary: AttendanceDailySummary | null;
}

export function ClaimOtDialog({ open, onOpenChange, summary }: ClaimOtDialogProps) {
  const { t } = useTranslation();
  const { submitClaim, isSubmitting } = useSubmitOtClaim();

  const [minutes, setMinutes] = useState(0);
  const [reason, setReason] = useState("");
  const [error, setError] = useState<string | null>(null);

  // Reset form when dialog opens
  useEffect(() => {
    if (!open) return; // Only run when opening

    if (summary) {
      setMinutes(summary.system_detected_ot_minutes || 0);
      setReason("");
      setError(null);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]); // Intentionally only depend on open, not summary

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!summary) return;
    
    if (minutes <= 0) {
        setError(t("ess.ot.error_zero_minutes") || "Minutes must be greater than 0");
        return;
    }
    
    if (minutes > summary.system_detected_ot_minutes) {
        setError(t("ess.ot.error_exceeds_system") || "Cannot claim more than system detected OT");
        return;
    }

    try {
      await submitClaim({
        date: summary.date,
        minutes,
        reason,
      });
      onOpenChange(false);
    } catch (e) {
      console.error(e);
      setError(t("common.error_occurred") || "An error occurred");
    }
  };

  if (!summary) return null;

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm animate-in fade-in duration-200" />
        <Dialog.Content className="fixed left-[50%] top-[50%] z-50 w-full max-w-md translate-x-[-50%] translate-y-[-50%] gap-4 border border-slate-100 bg-white p-6 shadow-premium-2xl duration-200 animate-in fade-in-0 zoom-in-95 slide-in-from-left-1/2 slide-in-from-top-48 rounded-[2rem]">
          <div className="flex flex-col gap-2 text-center sm:text-left">
            <Dialog.Title className="text-xl font-bold text-slate-800 uppercase tracking-tight">
              {t("ess.ot.claim_title") || "Claim Overtime"}
            </Dialog.Title>
            <Dialog.Description className="text-sm font-medium text-slate-500 uppercase tracking-wide">
              {t("ess.ot.claim_desc") || `For date: ${summary.date}`}
            </Dialog.Description>
          </div>
          
          <button 
            onClick={() => onOpenChange(false)}
            className="absolute right-4 top-4 rounded-full p-2 opacity-70 transition-opacity hover:opacity-100 hover:bg-slate-100"
          >
            <X className="h-4 w-4" />
            <span className="sr-only">Close</span>
          </button>

          <form onSubmit={handleSubmit} className="mt-6 space-y-6">
            <div className="bg-amber-50 border border-amber-100 rounded-xl p-4 flex items-start gap-3">
                <AlertCircle className="w-5 h-5 text-amber-600 mt-0.5" />
                <div>
                    <p className="text-xs font-bold text-amber-800 uppercase tracking-wide">System Detected</p>
                    <p className="text-lg font-bold text-amber-900">{summary.system_detected_ot_minutes} min</p>
                </div>
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                {t("ess.ot.claimed_minutes") || "Minutes to Claim"}
              </label>
              <div className="relative">
                <Clock className="absolute left-3 top-3 h-4 w-4 text-slate-400" />
                <input
                  type="number"
                  value={minutes}
                  onChange={(e) => setMinutes(parseInt(e.target.value) || 0)}
                  className="w-full pl-10 h-10 rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 text-sm ring-offset-white file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-slate-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-600 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 font-mono"
                  max={summary.system_detected_ot_minutes}
                />
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                {t("common.reason")}
              </label>
              <textarea
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                required
                className="w-full min-h-[80px] rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 text-sm ring-offset-white placeholder:text-slate-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-600 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 resize-none"
                placeholder={t("ess.ot.reason_placeholder") || "Why did you work overtime?"}
              />
            </div>

            {error && (
                <div className="text-xs font-bold text-rose-500 uppercase tracking-wide bg-rose-50 p-2 rounded-lg">
                    {error}
                </div>
            )}

            <div className="flex justify-end gap-3 pt-2">
              <button
                type="button"
                onClick={() => onOpenChange(false)}
                className="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider text-slate-600 hover:bg-slate-100 transition-colors"
              >
                {t("common.cancel")}
              </button>
              <button
                type="submit"
                disabled={isSubmitting}
                className="px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider shadow-lg shadow-primary-600/30 hover:bg-primary-700 hover:shadow-xl hover:-translate-y-0.5 transition-all disabled:opacity-50 disabled:pointer-events-none"
              >
                {isSubmitting ? t("common.saving") : t("common.submit")}
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
