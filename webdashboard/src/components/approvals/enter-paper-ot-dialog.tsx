import type { TFunction } from "i18next";

import * as Dialog from "@radix-ui/react-dialog";
import { X, Upload, FileText, Calendar, AlertCircle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { EmployeeSearch } from "@/components/common/employee-search";
import { Typography } from "@/components/ui/typography";
import { useEnterPaperOt } from "@/hooks/use-attendance";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

interface Props {
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  open: boolean;
}

interface EnterPaperOtFormProps {
  date: string;
  employeeId: string;
  error: string | null;
  handleSubmit: (e: React.SyntheticEvent) => void;
  isSubmitting: boolean;
  minutes: number;
  onCancel: () => void;
  reason: string;
  setDate: (date: string) => void;
  setEmployeeId: (id: string) => void;
  setMinutes: (minutes: number) => void;
  setReason: (reason: string) => void;
  t: TFunction;
  wsId: string;
}

function EmployeeSelector({ t, wsId, employeeId, setEmployeeId }: { employeeId: string; setEmployeeId: (id: string) => void; t: TFunction; wsId: string }) {
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
        {t("common.employee") || "Employee"}
      </label>
      <EmployeeSearch
        onChange={(value) => { setEmployeeId(Array.isArray(value) ? value[0] || "" : value); }}
        placeholder={t("approvals.paper_ot.select_employee") || "Search employee..."}
        value={employeeId}
        workspaceId={wsId}
      />
    </div>
  );
}

function DatePicker({ t, date, setDate }: { date: string; setDate: (date: string) => void; t: TFunction }) {
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
        {t("common.date") || "Date"}
      </label>
      <div className="relative">
        <Calendar className="absolute left-3 top-3 h-4 w-4 text-slate-400 dark:text-slate-500" />
        <input
          className="w-full pl-10 h-10 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-3 text-sm focus:ring-2 focus:ring-primary-600"
          max={format(new Date(), "yyyy-MM-dd")}
          onChange={(e) => { setDate(e.target.value); }}
          type="date"
          value={date}
        />
      </div>
    </div>
  );
}

function MinutesInput({ t, minutes, setMinutes }: { minutes: number; setMinutes: (m: number) => void; t: TFunction }) {
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
        {t("approvals.paper_ot.minutes") || "OT Minutes"}
      </label>
      <input
        className="w-full h-10 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-3 text-sm font-mono focus:ring-2 focus:ring-primary-600"
        min={1}
        onChange={(e) => { setMinutes(parseInt(e.target.value) || 0); }}
        placeholder="120"
        type="number"
        value={minutes || ""}
      />
    </div>
  );
}

function ReasonInput({ t, reason, setReason }: { reason: string; setReason: (r: string) => void; t: TFunction }) {
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
        {t("common.reason") || "Reason"}
      </label>
      <textarea
        className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-3 py-2 text-sm resize-none focus:ring-2 focus:ring-primary-600"
        onChange={(e) => { setReason(e.target.value); }}
        placeholder={t("approvals.paper_ot.reason_placeholder") || "Paper form #1234 - Production line"}
        rows={3}
        value={reason}
      />
    </div>
  );
}

function PhotoPlaceholder({ t }: { t: TFunction }) {
  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
        {t("approvals.paper_ot.photo") || "Paper Form Photo (Optional)"}
      </label>
      <div className="border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-xl p-4 text-center hover:border-primary-400 transition-colors cursor-not-allowed opacity-50">
        <Upload className="w-8 h-8 text-slate-400 dark:text-slate-500 mx-auto mb-2" />
        <Typography className="text-xs text-slate-500 dark:text-slate-400" variant="p">{t("approvals.paper_ot.photo_coming_soon") || "Photo upload coming soon"}</Typography>
      </div>
    </div>
  );
}

function FormActions({ t, isSubmitting, onCancel }: { isSubmitting: boolean; onCancel: () => void; t: TFunction }) {
  return (
    <div className="flex justify-end gap-3 pt-2">
      <button
        className="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
        onClick={onCancel}
        type="button"
      >
        {t("common.cancel") || "Cancel"}
      </button>
      <button
        className="px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-primary-700 disabled:opacity-50"
        disabled={isSubmitting}
        type="submit"
      >
        {isSubmitting ? (t("common.saving") || "Saving...") : (t("common.submit") || "Submit")}
      </button>
    </div>
  );
}

function EnterPaperOtForm({
  employeeId,
  setEmployeeId,
  date,
  setDate,
  minutes,
  setMinutes,
  reason,
  setReason,
  error,
  isSubmitting,
  handleSubmit,
  onCancel,
  wsId,
  t
}: EnterPaperOtFormProps) {
  return (
    <form className="mt-6 space-y-5" onSubmit={handleSubmit}>
      <EmployeeSelector employeeId={employeeId} setEmployeeId={setEmployeeId} t={t} wsId={wsId} />
      <DatePicker date={date} setDate={setDate} t={t} />
      <MinutesInput minutes={minutes} setMinutes={setMinutes} t={t} />
      <ReasonInput reason={reason} setReason={setReason} t={t} />
      <PhotoPlaceholder t={t} />

      {!!error && (
        <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3 animate-in fade-in slide-in-from-top-2 duration-300">
          <AlertCircle className="text-rose-600 dark:text-rose-300" size={18} />
          <Typography className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-widest" variant="p">{error}</Typography>
        </div>
      )}

      <FormActions isSubmitting={isSubmitting} onCancel={onCancel} t={t} />
    </form>
  );
}


export function EnterPaperOtDialog({ open, onOpenChange, onSuccess }: Props) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
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

  const validateForm = () => {
    if (!employeeId) {
      setError(t("approvals.paper_ot.error_no_employee") || "Select an employee");
      return false;
    }
    if (minutes <= 0) {
      setError(t("approvals.paper_ot.error_minutes") || "Minutes must be > 0");
      return false;
    }
    if (!reason.trim()) {
      setError(t("approvals.paper_ot.error_reason") || "Reason is required");
      return false;
    }
    return true;
  };

  const handleSubmit = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    setError(null);

    if (!validateForm()) return;

    try {
      await enterPaperOt({
        employeeId,
        date,
        claimed_minutes: minutes,
        reason,
        paper_form_photo_url: photoUrl ?? undefined,
      });
      resetForm();
      onOpenChange(false);
      onSuccess?.();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Failed to submit";
      setError(message);
    }
  };

  return (
    <Dialog.Root onOpenChange={(o) => { if (!o) resetForm(); onOpenChange(o); }} open={open}>
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

          <button className="absolute right-4 top-4 p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700" onClick={() => { onOpenChange(false); }}>
            <X className="h-4 w-4" />
          </button>

          <EnterPaperOtForm
            date={date}
            employeeId={employeeId}
            error={error}
            handleSubmit={handleSubmit}
            isSubmitting={isSubmitting}
            minutes={minutes}
            onCancel={() => { onOpenChange(false); }}
            reason={reason}
            setDate={setDate}
            setEmployeeId={setEmployeeId}
            setMinutes={setMinutes}
            setReason={setReason}
            t={t}
            wsId={wsId}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
