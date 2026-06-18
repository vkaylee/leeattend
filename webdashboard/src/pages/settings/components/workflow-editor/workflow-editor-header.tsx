import { ArrowLeft } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface WorkflowEditorHeaderProps {
  onBack: () => void;
  workflowId: string | undefined;
}

export function WorkflowEditorHeader({ workflowId, onBack }: WorkflowEditorHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <Button
          aria-label={t("common.back", { defaultValue: "Go back" })}
          className="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-colors"
          onClick={onBack}
          variant="ghost"
        >
          <ArrowLeft className="w-5 h-5 text-slate-600 dark:text-slate-300" />
        </Button>
        <div>
          <Typography className="font-bold mb-0.5 normal-case" variant="h3">
            {workflowId
              ? t("settings.workflows.edit_title", { defaultValue: "Edit Workflow" })
              : t("settings.workflows.new_title", { defaultValue: "New Workflow" })}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">
            {workflowId
              ? t("settings.workflows.edit_desc", { defaultValue: "Modify approval rules and steps" })
              : t("settings.workflows.new_desc", { defaultValue: "Define a new multi-step approval process" })}
          </Typography>
        </div>
      </div>
    </div>
  );
}
