import { Plus, ShieldAlert } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import type { BehalfAssignmentsEmptyStateProps, BehalfAssignmentsHeaderProps, AssignmentType } from "./types";

function getI18nKey(base: string, assignmentType: AssignmentType): string {
  if (assignmentType === "ot") return `${base}_ot`;
  if (assignmentType === "overtime_request") return `${base}_overtime_request`;
  if (assignmentType === "attendance_adjustment") return `${base}_adjustment`;
  return `${base}_leave`;
}

export function BehalfAssignmentsHeader({
  assignmentType,
  onAddClick,
}: BehalfAssignmentsHeaderProps) {
  const { t } = useTranslation();

  const title = t(getI18nKey("behalf_assignments.header.title", assignmentType));
  const description = t(getI18nKey("behalf_assignments.header.desc", assignmentType));

  return (
    <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-slate-50 dark:border-slate-800 pb-6">
      <div>
        <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
          {title}
        </Typography>
        <Typography className="text-xs text-slate-500 mt-1" variant="p">
          {description}
        </Typography>
      </div>
      <Button
        className="normal-case tracking-normal gap-2 h-10 px-6 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-bold text-sm shadow-md transition-all duration-300 transform active:scale-95"
        onClick={onAddClick}
        variant="primary"
      >
        <Plus className="w-4 h-4" />
        {t("behalf_assignments.header.add_btn")}
      </Button>
    </div>
  );
}

export function BehalfAssignmentsEmptyState({
  assignmentType,
}: BehalfAssignmentsEmptyStateProps) {
  const { t } = useTranslation();

  const title = t(getI18nKey("behalf_assignments.empty.title", assignmentType));
  const description = t(getI18nKey("behalf_assignments.empty.desc", assignmentType));

  return (
    <div className="py-24 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
      <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-6">
        <ShieldAlert className="text-slate-300 dark:text-slate-600" size={32} />
      </div>
      <Typography className="text-slate-700 dark:text-slate-100 font-extrabold text-xl tracking-tight" variant="h3">
        {title}
      </Typography>
      <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 max-w-sm" variant="p">
        {description}
      </Typography>
    </div>
  );
}
