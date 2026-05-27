import { Loader2, Check, X, Clock, Calendar } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useEmployeeConsent, useCancelOvertimeRequest } from "@/hooks/use-overtime-requests";
import { useToast } from "@/components/ui/toast";

export function MyOtAssignmentsList({ assignments, isLoading, wsId }: { assignments: any[]; isLoading: boolean; wsId: string }) {
  const { t } = useTranslation();
  const toast = useToast();
  const { mutate: consent, isPending: isConsenting } = useEmployeeConsent();
  const { mutate: cancel, isPending: isCanceling } = useCancelOvertimeRequest();

  const handleConsent = (id: string, employeeId: string, status: "accepted" | "declined") => {
    consent(
      { id, data: { employee_id: employeeId, status } },
      {
        onSuccess: () => {
          toast.success(status === "accepted" ? "Overtime accepted" : "Overtime declined");
        },
      }
    );
  };

  const handleCancel = (id: string) => {
    cancel(id, {
      onSuccess: () => {
        toast.success("Request canceled");
      }
    });
  };

  if (isLoading) {
    return (
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 p-12 flex justify-center shadow-sm">
        <Loader2 className="w-8 h-8 animate-spin text-slate-400" />
      </div>
    );
  }

  if (assignments.length === 0) {
    return (
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 p-16 text-center shadow-sm">
        <div className="w-16 h-16 bg-slate-50 dark:bg-slate-800 rounded-full flex items-center justify-center mx-auto mb-4">
          <Clock className="w-8 h-8 text-slate-300 dark:text-slate-600" />
        </div>
        <Typography className="text-lg font-bold text-slate-700 dark:text-slate-300">
          {t("ess.ot.no_assignments", { defaultValue: "No Overtime Assignments" })}
        </Typography>
        <Typography className="text-sm font-medium text-slate-500 mt-2">
          {t("ess.ot.no_assignments_desc", { defaultValue: "You don't have any pending overtime requests or assignments." })}
        </Typography>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {assignments.map((assignment) => {
        const isRequest = assignment.source_type === "employee_request";
        const isAssignment = assignment.source_type === "manager_assignment";

        // Find the specific employee status from the request payload
        const empStatus = assignment.employee_statuses?.find((s: any) => s.employee_id === assignment.my_employee_id);
        const currentConsent = empStatus?.status || "pending";

        return (
          <div key={assignment.id} className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-6 flex flex-col md:flex-row gap-6 justify-between shadow-sm hover:shadow-md transition-shadow">
            <div className="space-y-4 flex-1">
              <div className="flex items-center gap-3">
                <span className={`px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest ${
                  assignment.status === "approved" ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400" :
                  assignment.status === "rejected" ? "bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-400" :
                  assignment.status === "canceled" ? "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-400" :
                  "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
                }`}>
                  {assignment.status.replace("_", " ")}
                </span>
                {isAssignment && (
                  <span className="px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400">
                    Assignment
                  </span>
                )}
                {isRequest && (
                  <span className="px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-400">
                    My Request
                  </span>
                )}
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center gap-2 text-sm font-bold text-slate-700 dark:text-slate-300">
                  <Calendar className="w-4 h-4 text-slate-400" />
                  {new Date(assignment.planned_date).toLocaleDateString()}
                </div>
                <div className="flex items-center gap-2 text-sm font-bold text-slate-700 dark:text-slate-300">
                  <Clock className="w-4 h-4 text-slate-400" />
                  {assignment.planned_start_time} - {assignment.planned_end_time}
                </div>
              </div>

              {assignment.reason && (
                <Typography className="text-sm text-slate-600 dark:text-slate-400">
                  {assignment.reason}
                </Typography>
              )}
            </div>

            <div className="flex flex-col items-end justify-center gap-2 shrink-0">
              {isAssignment && currentConsent === "pending" && assignment.status !== "canceled" && (
                <div className="flex gap-2">
                  <button
                    className="flex items-center gap-1.5 bg-emerald-50 text-emerald-600 hover:bg-emerald-100 dark:bg-emerald-900/20 dark:hover:bg-emerald-900/40 px-4 py-2 rounded-xl font-bold text-xs transition-colors"
                    onClick={() => handleConsent(assignment.id, assignment.my_employee_id, "accepted")}
                    disabled={isConsenting}
                  >
                    <Check className="w-3.5 h-3.5" />
                    Accept
                  </button>
                  <button
                    className="flex items-center gap-1.5 bg-rose-50 text-rose-600 hover:bg-rose-100 dark:bg-rose-900/20 dark:hover:bg-rose-900/40 px-4 py-2 rounded-xl font-bold text-xs transition-colors"
                    onClick={() => handleConsent(assignment.id, assignment.my_employee_id, "declined")}
                    disabled={isConsenting}
                  >
                    <X className="w-3.5 h-3.5" />
                    Decline
                  </button>
                </div>
              )}

              {isAssignment && currentConsent !== "pending" && (
                <span className={`text-xs font-bold px-3 py-1.5 rounded-xl ${
                  currentConsent === "accepted" ? "text-emerald-600 bg-emerald-50 dark:text-emerald-400 dark:bg-emerald-900/20" : "text-rose-600 bg-rose-50 dark:text-rose-400 dark:bg-rose-900/20"
                }`}>
                  You {currentConsent}
                </span>
              )}

              {isRequest && assignment.status === "draft" && (
                <button
                  className="flex items-center gap-1.5 text-rose-500 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/20 px-4 py-2 rounded-xl font-bold text-xs transition-colors"
                  onClick={() => handleCancel(assignment.id)}
                  disabled={isCanceling}
                >
                  <X className="w-3.5 h-3.5" />
                  Cancel Request
                </button>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
