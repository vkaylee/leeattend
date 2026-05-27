import { Loader2, Clock, Plus, Users, AlertTriangle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

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

  const { employees } = useAllEmployees(workspace?.id || "");
  const { mutate: createRequest, isPending } = useCreateOvertimeRequest();

  // For simplicity, just fetch quota warning for the first selected employee to demonstrate the capability.
  // A production app would fetch a batch quota warning from backend.
  const sampleEmployeeId = selectedEmployees.length > 0 ? selectedEmployees[0] : "";
  const { data: quotaWarning, isLoading: isLoadingQuota } = useEmployeeQuota(sampleEmployeeId, plannedDate);

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
      toast.error("Please select at least one employee");
      return;
    }
    if (!reason.trim()) {
      toast.error("Please provide a reason");
      return;
    }

    const minutes = calculateMinutes();
    if (minutes <= 0) {
      toast.error("Invalid time range");
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
          toast.success("Pre-OT Request created successfully!");
          setOpen(false);
          onSuccess?.();
          // Reset
          setSelectedEmployees([]);
          setReason("");
        },
        onError: (err: any) => {
          toast.error(err?.response?.data?.error || "Failed to create OT request");
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
      <button 
        className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2.5 rounded-xl font-bold text-sm shadow-sm transition-all duration-300 hover:shadow hover:-translate-y-0.5"
        onClick={() => { setOpen(true); }}
      >
        <Plus className="w-4 h-4" />
        <span>New Pre-OT Request</span>
      </button>

      <Modal
        icon={<Clock className="w-5 h-5" />}
        isOpen={open}
        maxWidth="lg"
        onClose={() => { setOpen(false); }}
        subtitle="Assign overtime to employees beforehand"
        title="Plan Overtime (Pre-OT)"
      >
        <div className="space-y-6">
          {/* Employee Selection */}
          <div className="space-y-3">
            <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest flex items-center gap-1.5">
              <Users className="w-3.5 h-3.5" />
              Select Employees
            </label>
            <div className="max-h-40 overflow-y-auto border border-slate-200 dark:border-slate-700 rounded-xl p-2 bg-slate-50 dark:bg-slate-800/50 custom-scrollbar">
              {employees?.map((emp) => (
                <label className="flex items-center gap-3 p-2 hover:bg-white dark:hover:bg-slate-800 rounded-lg cursor-pointer transition-colors" key={emp.id}>
                  <input
                    checked={selectedEmployees.includes(emp.id)}
                    className="w-4 h-4 text-indigo-600 border-slate-300 rounded focus:ring-indigo-600 dark:border-slate-600 dark:bg-slate-900"
                    onChange={() => { toggleEmployee(emp.id); }}
                    type="checkbox"
                  />
                  <div className="flex flex-col">
                    <span className="text-sm font-bold text-slate-700 dark:text-slate-200">{emp.full_name}</span>
                    <span className="text-[10px] text-slate-500">{emp.employee_id_number}</span>
                  </div>
                </label>
              ))}
              {(!employees || employees.length === 0) && (
                <div className="p-4 text-center text-sm text-slate-500">No employees found</div>
              )}
            </div>
            <div className="text-xs font-bold text-indigo-600 dark:text-indigo-400">
              {selectedEmployees.length} selected
            </div>
          </div>

          {/* Quota Warning Banner */}
          {selectedEmployees.length > 0 && !!quotaWarning && quotaWarning.monthly_limit !== null && (
            <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl p-4 flex items-start gap-3">
              <AlertTriangle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
              <div>
                <Typography className="text-sm font-bold text-amber-800 dark:text-amber-400">
                  Quota Warning
                </Typography>
                <Typography className="text-xs text-amber-700 dark:text-amber-500 mt-1">
                  Selected employee has used {Math.round((quotaWarning.monthly_used || 0) / 60)}h out of {quotaWarning.monthly_limit}h this month.
                </Typography>
              </div>
            </div>
          )}

          {/* Date & Time */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Date</label>
              <input
                className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow"
                onChange={(e) => { setPlannedDate(e.target.value); }}
                type="date"
                value={plannedDate}
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Start Time</label>
              <input
                className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow"
                onChange={(e) => { setPlannedStartTime(e.target.value); }}
                type="time"
                value={plannedStartTime}
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">End Time</label>
              <input
                className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow"
                onChange={(e) => { setPlannedEndTime(e.target.value); }}
                type="time"
                value={plannedEndTime}
              />
            </div>
          </div>

          <div className="bg-slate-50 dark:bg-slate-800/50 rounded-xl p-3 flex justify-between items-center border border-slate-100 dark:border-slate-700">
            <span className="text-xs font-bold text-slate-500 uppercase">Estimated Duration</span>
            <span className="text-sm font-black text-indigo-600 dark:text-indigo-400">
              {Math.round(calculateMinutes() / 60 * 10) / 10} hours
            </span>
          </div>

          {/* Reason */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Business Reason</label>
            <textarea
              className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm font-medium focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-shadow resize-none h-24 custom-scrollbar"
              onChange={(e) => { setReason(e.target.value); }}
              placeholder="Why is this overtime necessary?"
              value={reason}
            />
          </div>
        </div>

        <div className="px-6 py-5 bg-slate-50 dark:bg-slate-900/50 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-3">
          <button
            className="px-5 py-2.5 text-sm font-bold text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-800 rounded-xl transition-colors"
            disabled={isPending}
            onClick={() => { setOpen(false); }}
          >
            Cancel
          </button>
          <button
            className="flex items-center justify-center min-w-[140px] px-5 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-xl transition-all shadow-sm hover:shadow"
            disabled={isPending || selectedEmployees.length === 0}
            onClick={handleCreate}
          >
            {isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : "Submit Request"}
          </button>
        </div>
      </Modal>
    </>
  );
}
