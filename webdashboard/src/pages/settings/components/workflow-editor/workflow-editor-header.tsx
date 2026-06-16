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
          aria-label="Go back"
          className="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-colors"
          onClick={onBack}
          variant="ghost"
        >
          <ArrowLeft className="w-5 h-5 text-slate-600 dark:text-slate-300" />
        </Button>
        <div>
          <Typography className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight" variant="h1">
            {workflowId
              ? t("settings.workflows.edit_title", { defaultValue: "Edit Workflow" })
              : t("settings.workflows.new_title", { defaultValue: "New Workflow" })}
          </Typography>
          <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1" variant="p">
            {workflowId
              ? t("settings.workflows.edit_desc", { defaultValue: "Modify approval rules and steps" })
              : t("settings.workflows.new_desc", { defaultValue: "Define a new multi-step approval process" })}
          </Typography>
        </div>
      </div>
    </div>
  );
}
