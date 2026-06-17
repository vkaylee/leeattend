import { ArrowDown, ArrowUp, Trash2 } from "lucide-react";
import React from "react";
import { Control, UseFormRegister } from "react-hook-form";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { WorkflowFormValues } from "../../hooks/workflow-schemas";
import { StepApproverConfig } from "./step-approver-config";
import { StepBasicFields } from "./step-basic-fields";

interface StepHeaderProps {
  canMoveDown: boolean;
  canMoveUp: boolean;
  canRemove: boolean;
  index: number;
  onMoveDown: () => void;
  onMoveUp: () => void;
  onRemove: () => void;
}

function StepHeader({
  index,
  canMoveUp,
  canMoveDown,
  canRemove,
  onMoveUp,
  onMoveDown,
  onRemove,
}: StepHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex justify-between items-center mb-5 border-b border-slate-100 dark:border-slate-800 pb-3">
      <div className="flex items-center gap-2">
        <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200" variant="p">
          {t("settings.workflows.step_title", { defaultValue: "Step" })} {index + 1}
        </Typography>
      </div>

      <div className="flex items-center gap-1">
        <Button
          aria-label={t("settings.workflows.move_up", { defaultValue: "Move step up" })}
          className="p-1.5 text-slate-400 hover:text-blue-600 hover:bg-white dark:hover:bg-slate-800 rounded-lg transition-colors disabled:opacity-30 disabled:pointer-events-none"
          disabled={!canMoveUp}
          onClick={onMoveUp}
          size="icon"
          title={t("settings.workflows.move_up", { defaultValue: "Move Up" })}
          variant="ghost"
        >
          <ArrowUp className="w-4 h-4" />
        </Button>

        <Button
          aria-label={t("settings.workflows.move_down", { defaultValue: "Move step down" })}
          className="p-1.5 text-slate-400 hover:text-blue-600 hover:bg-white dark:hover:bg-slate-800 rounded-lg transition-colors disabled:opacity-30 disabled:pointer-events-none"
          disabled={!canMoveDown}
          onClick={onMoveDown}
          size="icon"
          title={t("settings.workflows.move_down", { defaultValue: "Move Down" })}
          variant="ghost"
        >
          <ArrowDown className="w-4 h-4" />
        </Button>

        {canRemove ? (
          <Button
            aria-label={t("settings.workflows.delete_step", { defaultValue: "Delete step" })}
            className="p-1.5 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/20 rounded-lg transition-colors"
            onClick={onRemove}
            size="icon"
            title={t("settings.workflows.delete_step", { defaultValue: "Delete Step" })}
            variant="ghost"
          >
            <Trash2 className="w-4 h-4" />
          </Button>
        ) : null}
      </div>
    </div>
  );
}

interface WorkflowStepCardProps {
  approverType: "department_manager" | "role" | "permission" | "specific_user";
  canMoveDown: boolean;
  canMoveUp: boolean;
  canRemove: boolean;
  control: Control<WorkflowFormValues>;
  index: number;
  memberOptions: { value: string; label: string; description?: string }[];
  onMoveDown: () => void;
  onMoveUp: () => void;
  onRemove: () => void;
  permissionOptions: { value: string; label: string }[];
  register: UseFormRegister<WorkflowFormValues>;
  roleOptions: { value: string; label: string }[];
}

export function WorkflowStepCard({
  index,
  control,
  register,
  approverType,
  roleOptions,
  permissionOptions,
  memberOptions,
  onMoveUp,
  onMoveDown,
  onRemove,
  canMoveUp,
  canMoveDown,
  canRemove,
}: WorkflowStepCardProps) {
  return (
    <div className="bg-slate-50/50 dark:bg-slate-900/50 border border-slate-200/60 dark:border-slate-800 rounded-3xl p-5 relative group hover:border-slate-300 dark:hover:border-slate-700 transition-all duration-200">
      <div className="absolute -left-[45px] top-6 w-8 h-8 rounded-full bg-blue-600 dark:bg-blue-600 border-4 border-white dark:border-slate-950 flex items-center justify-center text-[10px] font-bold text-white shadow-sm ring-1 ring-slate-100 dark:ring-slate-900 transition-transform group-hover:scale-105">
        {index + 1}
      </div>

      <StepHeader
        canMoveDown={canMoveDown}
        canMoveUp={canMoveUp}
        canRemove={canRemove}
        index={index}
        onMoveDown={onMoveDown}
        onMoveUp={onMoveUp}
        onRemove={onRemove}
      />

      <StepBasicFields index={index} register={register} />

      <div className="mt-5 pt-4 border-t border-slate-200/50 dark:border-slate-800/50">
        <StepApproverConfig
          approverType={approverType}
          control={control}
          index={index}
          memberOptions={memberOptions}
          permissionOptions={permissionOptions}
          register={register}
          roleOptions={roleOptions}
        />
      </div>
    </div>
  );
}
