import React from "react";
import { FieldErrors, UseFormRegister } from "react-hook-form";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { WorkflowFormValues } from "../../hooks/workflow-schemas";

interface GeneralConfigCardProps {
  entityType: string;
  errors: FieldErrors<WorkflowFormValues>;
  isPending: boolean;
  onBack: () => void;
  register: UseFormRegister<WorkflowFormValues>;
}

export function GeneralConfigCard({ register, errors, isPending, onBack, entityType }: GeneralConfigCardProps) {
  const { t } = useTranslation();

  const getEntityLabel = (type: string) => {
    if (type === "leave_request") {
      return t("settings.workflows.entity_types.leave_request", { defaultValue: "Leave Requests" });
    }
    if (type === "attendance_adjustment") {
      return t("settings.workflows.entity_types.attendance_adjustment", { defaultValue: "Attendance Adjustments" });
    }
    return t("settings.workflows.entity_types.ot_request", { defaultValue: "Overtime Requests" });
  };

  return (
    <div className="space-y-6">
      <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-6 shadow-sm space-y-6">
        <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider" variant="h4">
          {t("settings.workflows.general_section", { defaultValue: "General Config" })}
        </Typography>

        <div className="space-y-1.5">
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest" htmlFor="workflow-name">
            {t("settings.workflows.name_label", { defaultValue: "Name" })}
          </label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus-visible:ring-2 focus-visible:ring-blue-500/50"
            id="workflow-name"
            placeholder={t("settings.workflows.name_placeholder", { defaultValue: "e.g. Standard OT Approval" })}
            {...register("name")}
          />
          {errors.name ? (
            <Typography className="text-rose-500 text-xs mt-1" variant="p">
              {t(errors.name.message ?? "")}
            </Typography>
          ) : null}
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest">
            {t("settings.workflows.entity_label", { defaultValue: "Entity Type" })}
          </label>
          <div className="w-full bg-slate-100 dark:bg-slate-800 border border-slate-200/60 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm font-bold text-slate-700 dark:text-slate-300">
            {getEntityLabel(entityType)}
          </div>
          <input type="hidden" {...register("entity_type")} />
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest" htmlFor="workflow-description">
            {t("settings.workflows.desc_label", { defaultValue: "Description" })}
          </label>
          <textarea
            className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus-visible:ring-2 focus-visible:ring-blue-500/50 min-h-[100px]"
            id="workflow-description"
            placeholder={t("settings.workflows.desc_placeholder", { defaultValue: "Optional description" })}
            {...register("description")}
          />
        </div>

        <div className="flex items-center gap-3">
          <input
            className="w-4 h-4 text-blue-600 rounded border-slate-300 focus:ring-blue-500"
            id="is_active"
            type="checkbox"
            {...register("is_active")}
          />
          <label className="text-sm font-medium text-slate-700 dark:text-slate-300" htmlFor="is_active">
            {t("settings.workflows.active_label", { defaultValue: "Active Workflow" })}
          </label>
        </div>
      </div>

      <div className="flex gap-3">
        <Button
          className="flex-1 px-5 py-2.5 rounded-xl font-bold text-sm text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          onClick={onBack}
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
        <Button
          className="flex-1 px-5 py-2.5 rounded-xl font-bold text-sm bg-blue-600 hover:bg-blue-700 text-white shadow-sm transition-all disabled:opacity-50"
          disabled={isPending}
          type="submit"
          variant="primary"
        >
          {isPending ? t("common.saving") : t("common.save")}
        </Button>
      </div>
    </div>
  );
}
