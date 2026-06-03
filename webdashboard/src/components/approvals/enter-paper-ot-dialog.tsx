import type { TFunction } from "i18next";

import * as Dialog from "@radix-ui/react-dialog";
import { X, Upload, FileText, Calendar, AlertCircle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
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

function PhotoUploader({
  t,
  photoUrl,
  setPhotoUrl,
}: {
  t: TFunction;
  photoUrl: string | null;
  setPhotoUrl: (url: string | null) => void;
}) {
  const [fileName, setFileName] = useState<string | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setFileName(file.name);
      setPhotoUrl(`/uploads/mock_${file.name}`);
    }
  };

  return (
    <div className="space-y-2">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
        {t("approvals.paper_ot.photo") || "Paper Form Photo (Optional)"}
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
        <div className="relative border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-xl p-6 text-center hover:border-primary-400 dark:hover:border-primary-500 transition-colors group cursor-pointer">
          <input
            accept="image/*"
            className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            onChange={handleFileChange}
            type="file"
          />
          <Upload className="w-8 h-8 text-slate-400 dark:text-slate-500 mx-auto mb-2 group-hover:text-primary-500 transition-colors" />
          <Typography className="text-xs font-semibold text-slate-600 dark:text-slate-300" variant="p">
            {t("approvals.paper_ot.upload_prompt") || "Click or drag photo to upload"}
          </Typography>
        </div>
      )}
    </div>
  );
}

function FormActions({ t, isSubmitting, onCancel }: { isSubmitting: boolean; onCancel: () => void; t: TFunction }) {
  return (
    <div className="flex justify-end gap-3 pt-2">
      <Button
        className="h-auto px-4 py-2 rounded-xl text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
        onClick={onCancel}
        type="button"
        variant="ghost"
      >
        {t("common.cancel") || "Cancel"}
      </Button>
      <Button
        className="h-auto px-6 py-2 rounded-xl text-xs font-bold uppercase tracking-wider shadow-lg"
        disabled={isSubmitting}
        loading={isSubmitting}
        type="submit"
        variant="primary"
      >
        {isSubmitting ? (t("common.saving") || "Saving...") : (t("common.submit") || "Submit")}
      </Button>
    </div>
  );
}

interface EnterPaperOtFormProps {
  date: string;
  employeeId: string;
  error: string | null;
  handleSubmit: (e: React.SyntheticEvent) => void;
  isSubmitting: boolean;
  minutes: number;
  onCancel: () => void;
  photoUrl: string | null;
  reason: string;
  setDate: (date: string) => void;
  setEmployeeId: (id: string) => void;
  setMinutes: (minutes: number) => void;
  setPhotoUrl: (url: string | null) => void;
  setReason: (reason: string) => void;
  t: TFunction;
  wsId: string;
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
  photoUrl,
  setPhotoUrl,
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
      <PhotoUploader photoUrl={photoUrl} setPhotoUrl={setPhotoUrl} t={t} />

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
  const [photoUrl, setPhotoUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const resetForm = () => {
    setEmployeeId("");
    setDate(format(new Date(), "yyyy-MM-dd"));
    setMinutes(0);
    setReason("");
    setPhotoUrl(null);
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

          <Button
            className="absolute right-4 top-4 h-auto w-auto p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
            icon={<X className="h-4 w-4" />}
            onClick={() => { onOpenChange(false); }}
            size="icon"
            variant="ghost"
          />

          <EnterPaperOtForm
            date={date}
            employeeId={employeeId}
            error={error}
            handleSubmit={handleSubmit}
            isSubmitting={isSubmitting}
            minutes={minutes}
            onCancel={() => { onOpenChange(false); }}
            photoUrl={photoUrl}
            reason={reason}
            setDate={setDate}
            setEmployeeId={setEmployeeId}
            setMinutes={setMinutes}
            setPhotoUrl={setPhotoUrl}
            setReason={setReason}
            t={t}
            wsId={wsId}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

