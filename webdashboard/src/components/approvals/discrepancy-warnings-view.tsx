import { AlertTriangle, ArrowRight } from "lucide-react";
import React from "react";

import type { TFunction } from "i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import type { ClaimItem } from "./confirm-claims-dialog";

interface DiscrepancyWarningsViewProps {
  discrepancies: ClaimItem[];
  error: string | null;
  isSubmitting: boolean;
  onConfirm: () => void;
  onGoBack: () => void;
  t: TFunction;
}

export function DiscrepancyWarningsView({
  discrepancies,
  error,
  isSubmitting,
  onGoBack,
  onConfirm,
  t,
}: DiscrepancyWarningsViewProps) {
  return (
    <div className="mt-6 space-y-6">
      <div className="p-4 bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900 rounded-2xl flex items-start gap-3">
        <AlertTriangle className="text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" size={20} />
        <div className="space-y-1">
          <Typography className="text-xs font-bold text-amber-800 dark:text-amber-300 uppercase tracking-wider" variant="p">
            {t("ot_approvals.delegated_claim.warning.title")}
          </Typography>
          <Typography className="text-xs text-amber-700 dark:text-amber-400" variant="p">
            {t("ot_approvals.delegated_claim.warning.details")}
          </Typography>
        </div>
      </div>

      {/* Discrepancy List */}
      <div className="border border-slate-100 dark:border-slate-800 rounded-2xl max-h-[220px] overflow-y-auto custom-scrollbar divide-y divide-slate-100 dark:divide-slate-800">
        {discrepancies.map((c) => {
          const diff = c.claimed_minutes - c.computed_minutes;
          const diffSign = diff > 0 ? `+${diff}` : `${diff}`;
          return (
            <div className="p-4 flex items-center justify-between text-xs hover:bg-slate-50/50 dark:hover:bg-slate-800/10 transition-colors" key={`${c.employment_id}-${c.date}`}>
              <div>
                <Typography className="font-bold text-slate-800 dark:text-slate-200" variant="p">
                  {c.employee_name}
                </Typography>
                <Typography className="text-[10px] text-slate-400" variant="p">
                  {c.employee_id_number} • {c.date}
                </Typography>
              </div>
              <div className="flex items-center gap-3 font-mono">
                <span className="text-slate-500">
                  {c.computed_minutes}m
                </span>
                <ArrowRight className="text-slate-400" size={14} />
                <span className={cn(
                  "font-bold",
                  diff > 0 ? "text-emerald-600 dark:text-emerald-400" : "text-rose-600 dark:text-rose-400"
                )}>
                  {c.claimed_minutes}m ({diffSign}m)
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {!!error && (
        <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3">
          <Typography className="text-xs font-semibold text-rose-600 dark:text-rose-300" variant="p">
            {error}
          </Typography>
        </div>
      )}

      {/* Discrepancy actions */}
      <div className="flex justify-end gap-3 pt-2">
        <Button
          className="h-auto px-4 py-2 rounded-xl text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
          disabled={isSubmitting}
          onClick={onGoBack}
          type="button"
          variant="ghost"
        >
          {t("ot_approvals.delegated_claim.warning.btn_edit")}
        </Button>
        <Button
          className="h-auto px-6 py-2 rounded-xl text-xs font-bold uppercase tracking-wider shadow-lg bg-amber-600 hover:bg-amber-700 text-white border-amber-600"
          disabled={isSubmitting}
          loading={isSubmitting}
          onClick={onConfirm}
          type="button"
          variant="primary"
        >
          {t("ot_approvals.delegated_claim.warning.btn_confirm")}
        </Button>
      </div>
    </div>
  );
}
