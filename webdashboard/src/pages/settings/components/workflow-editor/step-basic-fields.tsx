import React from "react";
import { UseFormRegister } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { WorkflowFormValues } from "../../hooks/workflow-schemas";

interface StepBasicFieldsProps {
  index: number;
  register: UseFormRegister<WorkflowFormValues>;
}

export function StepBasicFields({ index, register }: StepBasicFieldsProps) {
  const { t } = useTranslation();

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div className="space-y-1">
        <label
          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
          htmlFor={`step-name-${index}`}
        >
          {t("settings.workflows.step_name_label", { defaultValue: "Step Name" })}
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
          {t("settings.workflows.approver_type_label", { defaultValue: "Approver Type" })}
        </label>
        <select
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
          id={`step-approver-${index}`}
          {...register(`steps.${index}.approver_type` as const)}
        >
          <option value="department_manager">
            {t("settings.workflows.approver_types.dept_manager", { defaultValue: "Dept Manager" })}
          </option>
          <option value="role">
            {t("settings.workflows.approver_types.role", { defaultValue: "Role" })}
          </option>
          <option value="permission">
            {t("settings.workflows.approver_types.permission", { defaultValue: "Permission" })}
          </option>
          <option value="specific_user">
            {t("settings.workflows.approver_types.specific_user", { defaultValue: "Specific User" })}
          </option>
        </select>
      </div>

      <div className="space-y-1">
        <label
          className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
          htmlFor={`step-action-${index}`}
        >
          {t("settings.workflows.action_label", { defaultValue: "Action" })}
        </label>
        <select
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
          id={`step-action-${index}`}
          {...register(`steps.${index}.action_required` as const)}
        >
          <option value="approve">
            {t("settings.workflows.actions.approve", { defaultValue: "Approve" })}
          </option>
          <option value="review">
            {t("settings.workflows.actions.review", { defaultValue: "Review" })}
          </option>
          <option value="acknowledge">
            {t("settings.workflows.actions.acknowledge", { defaultValue: "Acknowledge" })}
          </option>
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
          {t("settings.workflows.can_skip_label", { defaultValue: "Can Skip if Not Found" })}
        </label>
      </div>
    </div>
  );
}
