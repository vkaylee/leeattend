import { Plus } from "lucide-react";
import React from "react";
import { Control, FieldArrayWithId, UseFormRegister } from "react-hook-form";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { WorkflowFormValues } from "../../hooks/workflow-schemas";
import { WorkflowStepCard } from "./workflow-step-card";

interface WorkflowStepsTimelineProps {
  append: (value: WorkflowFormValues["steps"][number]) => void;
  control: Control<WorkflowFormValues>;
  fields: FieldArrayWithId<WorkflowFormValues, "steps">[];
  memberOptions: { value: string; label: string; description?: string }[];
  move: (from: number, to: number) => void;
  permissionOptions: { value: string; label: string }[];
  register: UseFormRegister<WorkflowFormValues>;
  remove: (index: number) => void;
  roleOptions: { value: string; label: string }[];
  watchSteps: WorkflowFormValues["steps"];
}

export function WorkflowStepsTimeline({
  control,
  fields,
  append,
  move,
  remove,
  register,
  watchSteps,
  memberOptions,
  permissionOptions,
  roleOptions,
}: WorkflowStepsTimelineProps) {
  const { t } = useTranslation();

  return (
    <div className="lg:col-span-8 bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-6 shadow-sm space-y-6">
      <div className="flex justify-between items-center border-b border-slate-100 dark:border-slate-800 pb-4">
        <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider" variant="h4">
          {t("settings.workflows.steps_header", { defaultValue: "Approval Steps" })}
        </Typography>
        <Button
          className="flex items-center gap-1.5 text-xs font-bold text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 dark:bg-blue-900/20 dark:hover:bg-blue-900/30 px-4 py-2 rounded-xl transition-colors"
          onClick={() => {
            append({
              step_order: fields.length + 1,
              step_name: t("settings.workflows.step_default_name", { index: fields.length + 1, defaultValue: `Step ${fields.length + 1}` }),
              approver_type: "department_manager",
              role_id: "",
              permission_name: "",
              user_id: "",
              action_required: "approve",
              can_skip: false,
            });
          }}
        >
          <Plus className="w-4 h-4" />
          {t("settings.workflows.add_step", { defaultValue: "Add Step" })}
        </Button>
      </div>

      <div className="relative border-l border-slate-200 dark:border-slate-800 ml-4 pl-8 space-y-8">
        {fields.map((field, index) => {
          const approverType = watchSteps[index]?.approver_type ?? "department_manager";

          return (
            <WorkflowStepCard
              approverType={approverType}
              canMoveDown={index < fields.length - 1}
              canMoveUp={index > 0}
              canRemove={fields.length > 1}
              control={control}
              index={index}
              key={field.id}
              memberOptions={memberOptions}
              onMoveDown={() => { move(index, index + 1); }}
              onMoveUp={() => { move(index, index - 1); }}
              onRemove={() => { remove(index); }}
              permissionOptions={permissionOptions}
              register={register}
              roleOptions={roleOptions}
            />
          );
        })}
      </div>
    </div>
  );
}
