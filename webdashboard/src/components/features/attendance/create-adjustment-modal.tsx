import { useMutation, useQueryClient } from "@tanstack/react-query";
import { History, Info, Loader2, X } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { FileUpload } from "@/components/files/file-upload";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useBehalfEmployees } from "@/hooks/use-behalf-employees";
import { submitAttendanceAdjustment } from "@/lib/api";
import { toUTCISO } from "@/lib/datetime";

export interface CreateAdjustmentModalProps {
  isOpen: boolean;
  onClose: () => void;
  timezone: string;
  wsId: string;
}

interface CreateAdjustmentFormProps {
  onClose: () => void;
  timezone: string;
  wsId: string;
}

function useCreateAdjustmentForm({ wsId, timezone, onClose }: CreateAdjustmentFormProps) {
  const { t } = useTranslation(["common", "attendance"]);
  const toast = useToast();
  const queryClient = useQueryClient();

  const [selectedEmpId, setSelectedEmpId] = useState<string>("");
  const [date, setDate] = useState<string>(format(new Date(), "yyyy-MM-dd"));
  const [punchTime, setPunchTime] = useState<string>("08:00");
  const [reason, setReason] = useState<string>("");
  const [attachmentFileIds, setAttachmentFileIds] = useState<string[]>([]);
  const [ownerId] = useState<string>(() => crypto.randomUUID());

  const handleClose = () => {
    setSelectedEmpId("");
    setDate(format(new Date(), "yyyy-MM-dd"));
    setPunchTime("08:00");
    setReason("");
    setAttachmentFileIds([]);
    onClose();
  };

  const submitMutation = useMutation({
    mutationFn: (data: {
      employment_id: string;
      reason?: string;
      requested_punch_time: string;
      attachment_file_ids?: string[];
    }) => submitAttendanceAdjustment(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] });
      void queryClient.invalidateQueries({ queryKey: ["time-history"] });
      toast.success(`${t("time_tracking.modal.new_title")} ${t("common.success").toLowerCase()}`);
      handleClose();
    },
    onError: (e: unknown) => {
      const err = e as { message?: string; response?: { data?: { error?: string } } };
      toast.error(err.response?.data?.error ?? err.message ?? t("common.error"));
    },
  });

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!selectedEmpId) {
      toast.error(t("attendance_logs.select_employee"));
      return;
    }

    try {
      const punchTimeUtc = toUTCISO(date, punchTime, timezone);

      submitMutation.mutate({
        employment_id: selectedEmpId,
        requested_punch_time: punchTimeUtc,
        reason,
        attachment_file_ids: attachmentFileIds,
      });
    } catch (e: unknown) {
      const err = e as { message?: string };
      toast.error(err.message ?? t("common.error"));
    }
  };

  return {
    date,
    handleClose,
    handleSubmit,
    isPending: submitMutation.isPending,
    punchTime,
    reason,
    selectedEmpId,
    setDate,
    setPunchTime,
    setReason,
    setSelectedEmpId,
    attachmentFileIds,
    setAttachmentFileIds,
    ownerId,
  };
}

export function CreateAdjustmentModal({
  isOpen,
  onClose,
  timezone,
  wsId,
}: CreateAdjustmentModalProps) {
  const { t } = useTranslation(["common", "attendance"]);
  const { employees: behalfEmployees, isLoading: behalfLoading, hasAssignments } = useBehalfEmployees("attendance_adjustment");
  const {
    date,
    handleClose,
    handleSubmit,
    isPending,
    punchTime,
    reason,
    selectedEmpId,
    setDate,
    setPunchTime,
    setReason,
    setSelectedEmpId,
    attachmentFileIds,
    setAttachmentFileIds,
    ownerId,
  } = useCreateAdjustmentForm({ onClose, timezone, wsId });

  const behalfOptions = behalfEmployees
    .filter(emp => !!emp.active_employment_id)
    .map(emp => ({
      value: emp.active_employment_id!,
      label: `${emp.full_name} (${emp.employee_id_number})`,
    }));

  return (
    <Modal
      icon={<History size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={handleClose}
      subtitle={t("time_tracking.modal.subtitle")}
      title={t("time_tracking.modal.new_title")}
    >
      <form className="space-y-8 pt-4" onSubmit={handleSubmit}>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="adj-emp">
            {t("common.employee")}
          </label>
          {behalfLoading ? (
            <div className="h-14 rounded-2xl bg-slate-100 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 px-5 flex items-center gap-2 text-slate-400 text-sm">
              <Loader2 className="animate-spin" size={14} />
              {t("common.loading")}
            </div>
          ) : hasAssignments ? (
            <Combobox
              className="h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 shadow-sm"
              id="adj-emp"
              onChange={setSelectedEmpId}
              options={behalfOptions}
              placeholder={t("attendance_logs.select_employee")}
              value={selectedEmpId}
            />
          ) : (
            <div className="h-14 rounded-2xl bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/50 px-5 flex items-center gap-2 text-amber-700 dark:text-amber-400 text-sm">
              <Info size={14} />
              {t("behalf.no_assignments_configured", "Chưa có cấu hình behalf assignment")}
            </div>
          )}
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="adj-date">
            {t("common.work_day")}
          </label>
          <input
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight text-slate-700 dark:text-slate-200"
            id="adj-date"
            onChange={(e) => { setDate(e.target.value); }}
            required
            type="date"
            value={date}
          />
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="adj-punch-time">
            {t("attendance.adjustment.punch_time", "Giờ chấm công")}
          </label>
          <input
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center text-slate-700 dark:text-slate-200"
            id="adj-punch-time"
            onChange={(e) => { setPunchTime(e.target.value); }}
            required
            type="time"
            value={punchTime}
          />
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="adj-reason">
            {t("time_tracking.modal.reason_label")}
          </label>
          <textarea
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-medium text-sm h-28 resize-none text-slate-700 dark:text-slate-200"
            id="adj-reason"
            onChange={(e) => { setReason(e.target.value); }}
            placeholder={t("time_tracking.modal.reason_placeholder")}
            required
            value={reason}
          />
        </div>

        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {t("attendance.adjustment.attachment", "ATTACHMENT (PDF ONLY)")}
          </label>
          <FileUpload
            accept="application/pdf"
            category="AttendanceAdjustmentAttachment"
            maxSizeMB={20}
            onUploadSuccess={(file) => {
              setAttachmentFileIds([...attachmentFileIds, file.id]);
            }}
            ownerId={ownerId}
            ownerType="AttendanceAdjustment"
            workspaceId={wsId}
          />
          {attachmentFileIds.length > 0 && (
            <div className="mt-3 space-y-2">
              {attachmentFileIds.map((id, index) => (
                <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/50 rounded-xl border border-slate-105 dark:border-slate-800 text-xs animate-in slide-in-from-top-1" key={id}>
                  <span className="font-mono text-slate-700 dark:text-slate-350">Document #{index + 1} ({id})</span>
                  <Button
                    className="p-1 h-6 w-6 text-rose-600 hover:bg-rose-100 dark:hover:bg-rose-900/50 rounded-lg transition-colors border-none"
                    onClick={() => {
                      setAttachmentFileIds(attachmentFileIds.filter(fId => fId !== id));
                    }}
                    type="button"
                    variant="ghost"
                  >
                    <X className="w-4 h-4" />
                  </Button>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="pt-6 grid grid-cols-2 gap-5 border-t border-slate-50 dark:border-slate-800">
          <Button
            className="px-8 py-4 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all h-auto"
            onClick={handleClose}
            type="button"
            variant="ghost"
          >
            {t("common.cancel").toUpperCase()}
          </Button>
          <Button
            className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50 h-auto"
            disabled={isPending}
            type="submit"
            variant="primary"
          >
            {isPending ? t("common.sending").toUpperCase() : t("common.submit_request").toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
