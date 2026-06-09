import { useMutation, useQueryClient } from "@tanstack/react-query";
import { History, Info, Loader2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

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
  const [punchIn, setPunchIn] = useState<string>("08:00");
  const [punchOut, setPunchOut] = useState<string>("17:00");
  const [isBreakOvertime, setIsBreakOvertime] = useState<boolean>(false);
  const [reason, setReason] = useState<string>("");

  const handleClose = () => {
    setSelectedEmpId("");
    setDate(format(new Date(), "yyyy-MM-dd"));
    setPunchIn("08:00");
    setPunchOut("17:00");
    setIsBreakOvertime(false);
    setReason("");
    onClose();
  };

  const submitMutation = useMutation({
    mutationFn: (data: {
      employment_id: string;
      is_break_overtime?: boolean;
      reason?: string;
      requested_punch_in_time: string;
      requested_punch_out_time?: string;
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
      const punchInTime = toUTCISO(date, punchIn, timezone);
      const punchOutTime = punchOut ? toUTCISO(date, punchOut, timezone) : undefined;

      submitMutation.mutate({
        employment_id: selectedEmpId,
        requested_punch_in_time: punchInTime,
        requested_punch_out_time: punchOutTime,
        is_break_overtime: isBreakOvertime,
        reason,
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
    isBreakOvertime,
    isPending: submitMutation.isPending,
    punchIn,
    punchOut,
    reason,
    selectedEmpId,
    setDate,
    setIsBreakOvertime,
    setPunchIn,
    setPunchOut,
    setReason,
    setSelectedEmpId,
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
    isBreakOvertime,
    isPending,
    punchIn,
    punchOut,
    reason,
    selectedEmpId,
    setDate,
    setIsBreakOvertime,
    setPunchIn,
    setPunchOut,
    setReason,
    setSelectedEmpId,
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

        <div className="grid grid-cols-2 gap-6">
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="adj-punch-in">
              {t("common.punch_in")}
            </label>
            <input
              className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center text-slate-700 dark:text-slate-200"
              id="adj-punch-in"
              onChange={(e) => { setPunchIn(e.target.value); }}
              required
              type="time"
              value={punchIn}
            />
          </div>
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="adj-punch-out">
              {t("common.punch_out")}
            </label>
            <input
              className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center text-slate-700 dark:text-slate-200"
              id="adj-punch-out"
              onChange={(e) => { setPunchOut(e.target.value); }}
              required
              type="time"
              value={punchOut}
            />
          </div>
        </div>

        <div className="flex items-center gap-4 py-2 bg-slate-50 dark:bg-slate-800/50 p-4 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
          <input
            checked={isBreakOvertime}
            className="w-6 h-6 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer"
            id="is_break_overtime"
            onChange={(e) => { setIsBreakOvertime(e.target.checked); }}
            type="checkbox"
          />
          <label
            className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer leading-tight"
            htmlFor="is_break_overtime"
          >
            {t("attendance_confirm.ot_label")}
          </label>
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
