import React from "react";
import { UseFormRegister } from "react-hook-form";

import { WorkflowFormValues } from "../../hooks/workflow-schemas";

interface StepBasicFieldsProps {
  index: number;
  register: UseFormRegister<WorkflowFormValues>;
}

export function StepBasicFields({ index, register }: StepBasicFieldsProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div className="space-y-1">
        <label
          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
          htmlFor={`step-name-${index}`}
        >
          Step Name
        </label>
        <input
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
          id={`step-name-${index}`}
          {...register(`steps.${index}.step_name` as const)}
        />
      </div>

      <div className="space-y-1">
        <label
          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
          htmlFor={`step-approver-${index}`}
        >
          Approver Type
        </label>
        <select
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
          id={`step-approver-${index}`}
          {...register(`steps.${index}.approver_type` as const)}
        >
          <option value="department_manager">Dept Manager</option>
          <option value="role">Role</option>
          <option value="permission">Permission</option>
          <option value="specific_user">Specific User</option>
        </select>
      </div>

      <div className="space-y-1">
        <label
          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
          htmlFor={`step-action-${index}`}
        >
          Action
        </label>
        <select
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
          id={`step-action-${index}`}
          {...register(`steps.${index}.action_required` as const)}
        >
          <option value="approve">Approve</option>
          <option value="review">Review</option>
          <option value="acknowledge">Acknowledge</option>
        </select>
      </div>

      <div className="flex items-center gap-2 pt-5">
        <input
          className="w-4 h-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500"
          id={`skip-${index}`}
          type="checkbox"
          {...register(`steps.${index}.can_skip` as const)}
        />
        <label
          className="text-xs font-semibold text-slate-600 dark:text-slate-400"
          htmlFor={`skip-${index}`}
        >
          Can Skip if Not Found
        </label>
      </div>
    </div>
  );
}
