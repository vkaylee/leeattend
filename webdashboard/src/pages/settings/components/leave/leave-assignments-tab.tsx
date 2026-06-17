import { Info, UserPlus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { Employee, Department } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import type { LeaveAssignment } from "../../hooks/use-leave-settings";

interface LeaveAssignmentsTabProps {
  assignments: LeaveAssignment[];
  departments: Department[];
  employees: Employee[];
  isLoading: boolean;
  onAssignClick: () => void;
}

export function LeaveAssignmentsTab({
  assignments,
  isLoading,
  onAssignClick,
  employees,
  departments,
}: LeaveAssignmentsTabProps) {
  const { t } = useTranslation();


  return (
    <div>
      <div className="mb-8 border-b border-slate-50 dark:border-slate-800 pb-6 flex justify-between items-end">
        <div>
          <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">
            {t("settings.leave.tabs.assignments")}
          </Typography>
          <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 mt-1 uppercase tracking-tight" variant="p">
            {t("settings.leave.tabs.assignments_desc")}
          </Typography>
        </div>
        <Button className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 shadow-lg shadow-primary-100"
          data-testid="assign-policy-btn"
          onClick={() => { onAssignClick(); }}
          variant="ghost"
        >
          <UserPlus className="w-4 h-4" />
          {t("settings.leave.actions.assign")}
        </Button>
      </div>

      <div className="bg-amber-50 dark:bg-amber-950 border border-amber-100 dark:border-amber-800 p-8 rounded-[2.5rem] flex items-start gap-6">
        <Info className="text-amber-600 dark:text-amber-300 w-6 h-6 mt-1 shrink-0" />
        <div className="space-y-2">
          <Typography className="font-bold text-amber-900 uppercase tracking-widest text-xs" variant="p">{t("common.important_note")}</Typography>
          <Typography className="text-sm text-amber-800 leading-relaxed" variant="p">
            {t("settings.leave.messages.assignment_note")}
          </Typography>
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm mt-8 animate-in slide-in-from-bottom-4 duration-700">
        <table className="w-full text-left border-collapse">
          <thead className="bg-slate-50 dark:bg-slate-800 border-b border-slate-100 dark:border-slate-800">
            <tr>
              <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.policy")}</th>
              <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.type")}</th>
              <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.assignee")}</th>
              <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.effective_date")}</th>
              <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("settings.leave.table.created_at")}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <tr><td className="p-8 text-center text-slate-400 dark:text-slate-500 italic font-bold text-xs uppercase tracking-widest" colSpan={5}>{t("settings.leave.messages.loading")}</td></tr>
            ) : assignments.length === 0 ? (
              <tr><td className="p-12 text-center text-slate-400 dark:text-slate-500 italic font-bold text-xs uppercase tracking-widest opacity-50" colSpan={5}>{t("settings.leave.messages.no_assignments")}</td></tr>
            ) : (
              assignments.map((a) => {
                const getAssigneeName = () => {
                  if (a.assignee_type === "workspace") {
                    return t("leave_requests.policy.assignee_types.workspace");
                  }
                  if (a.assignee_type === "department") {
                    const dept = departments.find((d) => d.id === a.assignee_id);
                    return dept ? dept.name : (a.assignee_id ? `${a.assignee_id.slice(0, 8)}…` : t("common.unknown"));
                  }
                  if (a.assignee_type === "employee") {
                    const emp = employees.find((e) => e.id === a.assignee_id);
                    return emp ? emp.full_name : (a.assignee_id ? `${a.assignee_id.slice(0, 8)}…` : t("common.unknown"));
                  }
                  return t("common.unknown");
                };

                return (
                  <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group" key={a.id}>
                    <td className="px-6 py-5 font-bold text-slate-700 dark:text-slate-200 text-xs tracking-tight">{a.policy_name}</td>
                    <td className="px-6 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{a.leave_type_name}</td>
                    <td className="px-6 py-5">
                      <div className="flex items-center gap-3">
                        <span className={cn("px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border shadow-sm shrink-0", 
                          a.assignee_type === "workspace" ? "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100" :
                          a.assignee_type === "department" ? "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800" :
                          "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                        )}>
                          {a.assignee_type}
                        </span>
                        {a.assignee_type !== "workspace" && (
                          <span className="font-bold text-slate-700 dark:text-slate-200 text-xs truncate max-w-[180px]">
                            {getAssigneeName()}
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="px-6 py-5 text-xs font-mono text-slate-600 dark:text-slate-300 font-bold">
                       {format(new Date(a.effective_from), "MMM dd, yyyy")} 
                       <span className="text-slate-300 dark:text-slate-600 mx-2">→</span>
                       {a.effective_to ? format(new Date(a.effective_to), "MMM dd, yyyy") : <span className="text-slate-400 dark:text-slate-500 italic">{t("common.forever")}</span>}
                    </td>
                    <td className="px-6 py-5 text-[10px] font-mono text-slate-400 dark:text-slate-500 text-right uppercase">
                      {format(new Date(a.created_at), "MMM dd HH:mm")}
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
