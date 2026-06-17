import { Clock, Milestone, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { LeavePolicy } from "@/lib/api/types";

interface PolicyCardProps {
  bonusCount: number;
  onDelete: (id: string) => void;
  onEdit: (policy: LeavePolicy) => void;
  policy: LeavePolicy;
  typeName: string;
}

export function PolicyCard({ policy, typeName, bonusCount, onEdit, onDelete }: PolicyCardProps) {
  const { t } = useTranslation();
  return (
    <div className="relative flex flex-col justify-between rounded-[2rem] border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-7 shadow-sm hover:shadow-md transition-all group" key={policy.id}>
      <div>
        <div className="flex items-center justify-between">
          <span className="inline-flex items-center rounded-lg bg-primary-50 dark:bg-primary-950 px-2.5 py-1 text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest border border-primary-100 dark:border-primary-800 shadow-sm">
            {t(`settings.leave.templates.${typeName.toLowerCase().replace(/ /g, "-")}.name`, typeName)}
          </span>
          <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button 
              className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800" 
              data-testid="edit-policy-btn" 
              onClick={() => { onEdit(policy); }}
              size="icon"
              variant="outline"
            >
              <Pencil size={16} />
            </Button>
            <Button 
              className="h-9 w-9 p-2 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800" 
              data-testid="delete-policy-btn" 
              onClick={() => { onDelete(policy.id); }}
              size="icon"
              variant="outline"
            >
              <Trash2 size={16} />
            </Button>
          </div>
        </div>
        <Typography className="mt-5 text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight leading-none uppercase" variant="h3">{policy.name}</Typography>
        
        <div className="mt-6 p-4 bg-slate-50 dark:bg-slate-800 rounded-2xl border border-slate-100 dark:border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-slate-400 dark:text-slate-500">
              <Clock size={14} />
              <span className="text-[10px] font-bold uppercase tracking-widest">{t("settings.leave.policy_card.base_accrual")}</span>
            </div>
            <span className="text-[10px] font-bold text-slate-600 dark:text-slate-300 uppercase tracking-widest">
              {policy.rule_config.accrual_amount} / {t(`settings.leave.policy_fields.frequency_options.${policy.rule_config.frequency}`, policy.rule_config.frequency)}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-slate-400 dark:text-slate-500">
              <Milestone size={14} />
              <span className="text-[10px] font-bold uppercase tracking-widest">{t("settings.leave.policy_card.bonus_steps")}</span>
            </div>
            <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest">
              {t("settings.leave.policy_card.steps_count", { count: bonusCount })}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
