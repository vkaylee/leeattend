import { Users } from "lucide-react";
import React from "react";
import { Control, Controller, UseFormRegister } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";

import { WorkflowFormValues } from "../../hooks/workflow-schemas";

interface OptionConfigProps {
  index: number;
  options: { value: string; label: string }[];
  register: UseFormRegister<WorkflowFormValues>;
}

export function DeptManagerConfig() {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-3 bg-blue-50/50 dark:bg-blue-900/10 border border-blue-100/50 dark:border-blue-900/20 rounded-2xl p-4 text-xs text-blue-700 dark:text-blue-400">
      <Users className="w-5 h-5 shrink-0 text-blue-500" />
      <div>
        <Typography className="font-bold" variant="p">
          {t("settings.workflows.dept_manager_routing", { defaultValue: "Department Manager Routing" })}
        </Typography>
        <Typography className="mt-0.5 opacity-90" variant="p">
          {t("settings.workflows.dept_manager_routing_desc", {
            defaultValue: "Requests will be sent automatically to the manager of the requester's department.",
          })}
        </Typography>
      </div>
    </div>
  );
}

export function RoleConfig({ index, register, options }: OptionConfigProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-1 max-w-md">
      <label
        className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
        htmlFor={`step-role-${index}`}
      >
        {t("settings.workflows.select_role_label", { defaultValue: "Select Role" })}
      </label>
      <select
        className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
        id={`step-role-${index}`}
        {...register(`steps.${index}.role_id` as const)}
      >
        <option value="">
          {t("settings.workflows.choose_role_placeholder", { defaultValue: "Choose a Role..." })}
        </option>
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}

export function PermissionConfig({ index, register, options }: OptionConfigProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-1 max-w-md">
      <label
        className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
        htmlFor={`step-permission-${index}`}
      >
        {t("settings.workflows.select_permission_label", { defaultValue: "Select Permission" })}
      </label>
      <select
        className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3.5 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30"
        id={`step-permission-${index}`}
        {...register(`steps.${index}.permission_name` as const)}
      >
        <option value="">
          {t("settings.workflows.choose_permission_placeholder", { defaultValue: "Choose a Permission..." })}
        </option>
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}

interface SpecificUserConfigProps {
  control: Control<WorkflowFormValues>;
  index: number;
  options: { value: string; label: string; description?: string }[];
}

export function SpecificUserConfig({ index, control, options }: SpecificUserConfigProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-1.5 max-w-md">
      <label
        className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider"
        htmlFor={`step-specific-user-${index}`}
      >
        {t("settings.workflows.search_user_label", { defaultValue: "Search Workspace User" })}
      </label>
      <Controller
        control={control}
        name={`steps.${index}.user_id` as const}
        render={({ field }) => (
          <Combobox
            emptyText={t("settings.workflows.no_members_found", { defaultValue: "No members found" })}
            id={`step-specific-user-${index}`}
            onChange={field.onChange}
            options={options}
            placeholder={t("settings.workflows.search_user_placeholder", { defaultValue: "Type to search user..." })}
            searchPlaceholder={t("settings.workflows.search_members_placeholder", { defaultValue: "Search members..." })}
            value={field.value}
          />
        )}
      />
    </div>
  );
}

interface StepApproverConfigProps {
  approverType: "department_manager" | "role" | "permission" | "specific_user";
  control: Control<WorkflowFormValues>;
  index: number;
  memberOptions: { value: string; label: string; description?: string }[];
  permissionOptions: { value: string; label: string }[];
  register: UseFormRegister<WorkflowFormValues>;
  roleOptions: { value: string; label: string }[];
}

export function StepApproverConfig({
  index,
  control,
  register,
  approverType,
  roleOptions,
  permissionOptions,
  memberOptions,
}: StepApproverConfigProps) {
  switch (approverType) {
    case "department_manager":
      return <DeptManagerConfig />;
    case "role":
      return <RoleConfig index={index} options={roleOptions} register={register} />;
    case "permission":
      return <PermissionConfig index={index} options={permissionOptions} register={register} />;
    case "specific_user":
      return <SpecificUserConfig control={control} index={index} options={memberOptions} />;
    default:
      return null;
  }
}
