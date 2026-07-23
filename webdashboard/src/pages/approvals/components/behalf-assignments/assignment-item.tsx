import { ArrowRight, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

import type { AssignmentItemProps } from "./types";

import { TargetScopeDisplay } from "./target-scope-display";

export function AssignmentItem({
  assignment,
  assignmentType,
  departments,
  isDeleting,
  onDelete,
}: AssignmentItemProps) {
  const { t } = useTranslation();

  const assigneeLabel = t(`behalf_assignments.assignee_labels.${assignmentType}`);
  const targetLabel = t(`behalf_assignments.target_labels.${assignmentType}`);

  const targetEmp = assignment.target_employee_id
    ? {
        full_name: assignment.target_employee_name ?? t("behalf_assignments.unknown_employee"),
        employee_id_number: assignment.target_employee_id_number,
      }
    : null;

  const targetDept = assignment.target_department_id
    ? {
        name: assignment.target_department_name ?? t("behalf_assignments.unknown_department"),
        description: departments.find((d) => d.id === assignment.target_department_id)?.description ?? null,
      }
    : null;

  const assigneeName = assignment.assignee_name ?? t("behalf_assignments.unknown_employee");
  const assigneeIdNumber = assignment.assignee_employee_id_number;

  return (
    <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-100 dark:border-slate-800 p-5 flex items-center justify-between shadow-sm hover:shadow-md transition-all duration-200">
      <div className="flex flex-col md:flex-row md:items-center gap-4 md:gap-8 flex-1">
        <div className="flex flex-col min-w-[200px]">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            {assigneeLabel}
          </span>
          <span className="font-bold text-slate-800 dark:text-slate-200 text-sm mt-0.5">
            {assigneeName}
          </span>
          {assigneeIdNumber ? (
            <span className="text-[10px] text-slate-400 font-mono mt-0.5">
              {assigneeIdNumber}
            </span>
          ) : null}
        </div>

        <div className="flex items-center gap-2 text-slate-300 dark:text-slate-600 self-center hidden md:flex">
          <ArrowRight className="w-5 h-5" />
        </div>

        <div className="flex flex-col">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            {targetLabel}
          </span>
          <TargetScopeDisplay targetDept={targetDept} targetEmp={targetEmp} />
        </div>
      </div>

      <Button
        className="p-2 text-slate-400 hover:text-rose-600 dark:text-slate-600 dark:hover:text-rose-400 rounded-xl hover:bg-rose-50 dark:hover:bg-rose-950/20 transition-all border-none shadow-none h-10 w-10 flex items-center justify-center shrink-0"
        disabled={isDeleting}
        onClick={() => {
          void onDelete(assignment.id);
        }}
        variant="ghost"
      >
        <Trash2 className="w-4 h-4" />
      </Button>
    </div>
  );
}
