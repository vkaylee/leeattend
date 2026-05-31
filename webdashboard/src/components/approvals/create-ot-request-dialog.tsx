import { Clock, Plus, Users, AlertTriangle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useAllEmployees } from "@/hooks/use-employees";
import { useCreateOvertimeRequest, useEmployeeQuota } from "@/hooks/use-overtime-requests";
import { useWorkspace } from "@/hooks/use-workspace";

export function CreateOtRequestDialog({ onSuccess }: { onSuccess?: () => void }) {
  const { t } = useTranslation();
  const toast = useToast();
  const { workspace } = useWorkspace();
  
  const [open, setOpen] = useState(false);
  const [selectedEmployees, setSelectedEmployees] = useState<string[]>([]);
  const [plannedDate, setPlannedDate] = useState(format(new Date(), "yyyy-MM-dd"));
  const [plannedStartTime, setPlannedStartTime] = useState("17:00");
  const [plannedEndTime, setPlannedEndTime] = useState("19:00");
  const [reason, setReason] = useState("");

  const { employees } = useAllEmployees(workspace?.id ?? "");
  const { mutate: createRequest, isPending } = useCreateOvertimeRequest();

  // For simplicity, just fetch quota warning for the first selected employee to demonstrate the capability.
  // A production app would fetch a batch quota warning from backend.
  const sampleEmployeeId = selectedEmployees.length > 0 ? selectedEmployees[0] : "";
  const { data: quotaWarning } = useEmployeeQuota(sampleEmployeeId, plannedDate);

  const calculateMinutes = () => {
    if (!plannedStartTime || !plannedEndTime) return 0;
    const [startH, startM] = plannedStartTime.split(":").map(Number);
    const [endH, endM] = plannedEndTime.split(":").map(Number);
    let minutes = (endH * 60 + endM) - (startH * 60 + startM);
    if (minutes < 0) minutes += 24 * 60; // Crosses midnight
    return minutes;
  };

  const handleCreate = () => {
    if (selectedEmployees.length === 0) {
      toast.error(t("approvals.pre_ot_dialog.validation.select_employee"));
      return;
    }
    if (!reason.trim()) {
      toast.error(t("approvals.pre_ot_dialog.validation.provide_reason"));
      return;
    }

    const minutes = calculateMinutes();
    if (minutes <= 0) {
      toast.error(t("approvals.pre_ot_dialog.validation.invalid_time"));
      return;
    }

    createRequest(
      {
        planned_date: plannedDate,
        planned_start_time: plannedStartTime + ":00",
        planned_end_time: plannedEndTime + ":00",
        planned_minutes: minutes,
        reason,
        employees: selectedEmployees,
      },
      {
        onSuccess: () => {
          toast.success(t("approvals.pre_ot_dialog.success_message"));
          setOpen(false);
          onSuccess?.();
          // Reset
          setSelectedEmployees([]);
          setReason("");
        },
        onError: (err: unknown) => {
          const apiErr = err as { response?: { data?: { error?: string } } };
          toast.error(apiErr.response?.data?.error ?? t("approvals.pre_ot_dialog.error_message"));
        },
      }
    );
  };

  const toggleEmployee = (id: string) => {
    setSelectedEmployees(prev => 
      prev.includes(id) ? prev.filter(e => e !== id) : [...prev, id]
    );
  };

  return (
    <>
      <Button 
        className="normal-case tracking-normal h-10 px-4 rounded-xl gap-2 font-bold text-sm shadow-sm transition-all duration-300 hover:shadow hover:-translate-y-0.5"
        icon={<Plus className="w-4 h-4" />}
        onClick={() => { setOpen(true); }}
        variant="primary"
      >
        {t("approvals.pre_ot_dialog.new_request_btn")}
      </Button>

      <Modal
        icon={<Clock className="w-5 h-5" />}
        isOpen={open}
        maxWidth="lg"
        onClose={() => { setOpen(false); }}
        subtitle={t("approvals.pre_ot_dialog.modal_subtitle")}
        title={t("approvals.pre_ot_dialog.modal_title")}
      >
        <div className="space-y-6">
          {/* Employee Selection */}
          <div className="space-y-3">
            <div className="text-[10px] font-bold text-slate-500 uppercase tracking-widest flex items-center gap-1.5">
              <Users className="w-3.5 h-3.5" />
              {t("approvals.pre_ot_dialog.select_employees")}
            </div>
            <div className="max-h-40 overflow-y-auto border border-slate-200 dark:border-slate-700 rounded-xl p-2 bg-slate-50 dark:bg-slate-800/50 custom-scrollbar">
              {employees.map((emp) => (
                <div 
                  className="flex items-center gap-3 p-2 hover:bg-white dark:hover:bg-slate-800 rounded-lg transition-colors" 
                  key={emp.id}
                >
                  <input
                    checked={selectedEmployees.includes(emp.id)}
                    className="w-4 h-4 text-indigo-600 border-slate-300 rounded focus:ring-indigo-600 dark:border-slate-600 dark:bg-slate-900"
                    id={`emp-checkbox-${emp.id}`}
                    onChange={() => { toggleEmployee(emp.id); }}
                    type="checkbox"
                  />
                  <label 
                    className="flex flex-col cursor-pointer" 
                    htmlFor={`emp-checkbox-${emp.id}`}
                  >
                    <span className="text-sm font-bold text-slate-700 dark:text-slate-200">{emp.full_name}</span>
                    <span className="text-[10px] text-slate-500">{emp.employee_id_number}</span>
                  </label>
                </div>
              ))}
              {employees.length === 0 && (
                <div className="p-4 text-center text-sm text-slate-500">{t("approvals.pre_ot_dialog.no_employees")}</div>
              )}
            </div>
            <div className="text-xs font-bold text-indigo-600 dark:text-indigo-400">
              {t("approvals.pre_ot_dialog.selected_count", { count: selectedEmployees.length })}
            </div>
          </div>

          {/* Quota Warning Banner */}
          {selectedEmployees.length > 0 && !!quotaWarning && quotaWarning.monthly_limit !== null && (
            <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl p-4 flex items-start gap-3">
              <AlertTriangle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
              <div>
                <Typography className="text-sm font-bold text-amber-800 dark:text-amber-400">
                  {t("approvals.pre_ot_dialog.quota_warning_title")}
                </Typography>
                <Typography className="text-xs text-amber-700 dark:text-amber-500 mt-1">
                  {t("approvals.pre_ot_dialog.quota_warning_desc", {
                    limit: quotaWarning.monthly_limit,
                    used: Math.round(quotaWarning.monthly_used / 60),
                  })}
                </Typography>
              </div>
            </div>
          )}

          {/* Date & Time */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div className="space-y-1.5">
              <label 
                className="text-[10px] font-bold text-slate-500 uppercase tracking-widest" 
                htmlFor="planned-date"
              >
                {t("approvals.pre_ot_dialog.date")}
              </label>
              <input
                className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow"
                id="planned-date"
                onChange={(e) => { setPlannedDate(e.target.value); }}
                type="date"
                value={plannedDate}
              />
            </div>
            <div className="space-y-1.5">
              <label 
                className="text-[10px] font-bold text-slate-500 uppercase tracking-widest" 
                htmlFor="planned-start-time"
              >
                {t("approvals.pre_ot_dialog.start_time")}
              </label>
              <input
                className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow"
                id="planned-start-time"
                onChange={(e) => { setPlannedStartTime(e.target.value); }}
                type="time"
                value={plannedStartTime}
              />
            </div>
            <div className="space-y-1.5">
              <label 
                className="text-[10px] font-bold text-slate-500 uppercase tracking-widest" 
                htmlFor="planned-end-time"
              >
                {t("approvals.pre_ot_dialog.end_time")}
              </label>
              <input
                className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow"
                id="planned-end-time"
                onChange={(e) => { setPlannedEndTime(e.target.value); }}
                type="time"
                value={plannedEndTime}
              />
            </div>
          </div>

          <div className="bg-slate-50 dark:bg-slate-800/50 rounded-xl p-3 flex justify-between items-center border border-slate-100 dark:border-slate-700">
            <span className="text-xs font-bold text-slate-500 uppercase">{t("approvals.pre_ot_dialog.estimated_duration")}</span>
            <span className="text-sm font-black text-indigo-600 dark:text-indigo-400">
              {t("approvals.pre_ot_dialog.duration_hours", {
                hours: Math.round((calculateMinutes() / 60) * 10) / 10,
              })}
            </span>
          </div>

          {/* Reason */}
          <div className="space-y-1.5">
            <label 
              className="text-[10px] font-bold text-slate-500 uppercase tracking-widest" 
              htmlFor="business-reason"
            >
              {t("approvals.pre_ot_dialog.business_reason")}
            </label>
            <textarea
              className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow resize-none h-24 custom-scrollbar"
              id="business-reason"
              onChange={(e) => { setReason(e.target.value); }}
              placeholder={t("approvals.pre_ot_dialog.reason_placeholder")}
              value={reason}
            />
          </div>
        </div>

        <div className="px-6 py-5 bg-slate-50 dark:bg-slate-900/50 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-3">
          <Button
            className="normal-case tracking-normal px-5 py-2.5 h-10 rounded-xl"
            disabled={isPending}
            onClick={() => { setOpen(false); }}
            variant="ghost"
          >
            {t("common.cancel")}
          </Button>
          <Button
            className="normal-case tracking-normal min-w-[140px] px-5 py-2.5 h-10 rounded-xl"
            disabled={isPending || selectedEmployees.length === 0}
            loading={isPending}
            onClick={handleCreate}
            variant="primary"
          >
            {t("approvals.pre_ot_dialog.submit_btn")}
          </Button>
        </div>
      </Modal>
    </>
  );
}
