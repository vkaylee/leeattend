import { Users, Building, Globe, Trash2, Calendar, Sliders } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { PayrollRuleAssignment, Department } from "@/lib/api/types";
import { cn } from "@/lib/utils";

export const SCOPE_ICONS = {
  organization: Globe,
  department: Building,
  employee: Users,
} as const;

export const SCOPE_COLORS = {
  organization: {
    bg: "bg-amber-50 dark:bg-amber-950/40",
    text: "text-amber-600 dark:text-amber-400",
    border: "border-amber-100 dark:border-amber-800",
    badge: "bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300",
  },
  department: {
    bg: "bg-sky-50 dark:bg-sky-950/40",
    text: "text-sky-600 dark:text-sky-400",
    border: "border-sky-100 dark:border-sky-800",
    badge: "bg-sky-100 dark:bg-sky-900/50 text-sky-700 dark:text-sky-300",
  },
  employee: {
    bg: "bg-teal-50 dark:bg-teal-950/40",
    text: "text-teal-600 dark:text-teal-400",
    border: "border-teal-100 dark:border-teal-800",
    badge: "bg-teal-100 dark:bg-teal-900/50 text-teal-700 dark:text-teal-300",
  },
} as const;

interface AssignmentCardProps {
  a: PayrollRuleAssignment;
  deletingId: string | null;
  deptMap: Record<string, Department>;
  handleDelete: (id: string) => void;
  isDeleting: boolean;
  scope: "organization" | "department" | "employee";
  setDeletingId: (id: string | null) => void;
  t: TFunction;
}

// eslint-disable-next-line complexity -- Presentational logic
export function AssignmentCard({ 
  a, 
  deptMap, 
  t, 
  scope, 
  deletingId, 
  isDeleting, 
  setDeletingId, 
  handleDelete 
}: AssignmentCardProps) {
  const { formatDate } = useDateTimeFormatter();
  const colors = SCOPE_COLORS[scope];
  const Icon = SCOPE_ICONS[scope];

  const getAssigneeName = (assignment: PayrollRuleAssignment) => {
    if (assignment.assignee_type === "organization") {
      return t("payroll_rules.scope_organization", "Entire Organization");
    }
    if (assignment.assignee_type === "department") {
      const dept = assignment.assignee_id ? deptMap[assignment.assignee_id] : null;
      const deptName = dept ? dept.name : null;
      return deptName ?? (assignment.assignee_id ? `${assignment.assignee_id.slice(0, 8)}…` : "Unknown");
    }
    return assignment.assignee_id ? `${assignment.assignee_id.slice(0, 8)}…` : "Unknown";
  };

  const overrideConfigObj = a.override_config && typeof a.override_config === "object" ? (a.override_config as { rate?: unknown; amount?: unknown }) : null;
  const overrideRate = overrideConfigObj && 'rate' in overrideConfigObj && overrideConfigObj.rate != null ? Number(overrideConfigObj.rate) : null;
  const overrideAmount = overrideConfigObj && 'amount' in overrideConfigObj && overrideConfigObj.amount != null ? Number(overrideConfigObj.amount) : null;

  return (
    <div
      className={cn(
        "flex items-center justify-between p-4 rounded-xl border shadow-sm transition-all group hover:shadow-md",
        colors.bg,
        colors.border,
        deletingId === a.id && "ring-2 ring-rose-300 dark:ring-rose-700"
      )}
    >
      <div className="flex items-center gap-4 min-w-0">
        <div
          className={cn(
            "w-10 h-10 rounded-xl flex items-center justify-center shrink-0",
            colors.badge
          )}
        >
          <Icon size={18} />
        </div>
        <div className="min-w-0">
          <Typography className="font-bold text-sm text-slate-700 dark:text-slate-200 uppercase tracking-tight truncate leading-none" variant="p">
            {getAssigneeName(a)}
          </Typography>
          <div className="flex items-center gap-3 mt-1.5 flex-wrap">
            <span className="flex items-center gap-1 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
              <Calendar className="opacity-50" size={10} />
              {formatDate(a.effective_from)}
              {a.effective_to
                ? ` → ${formatDate(a.effective_to)}`
                : " → ∞"}
            </span>
            {!!a.override_config && (
              <span className="flex items-center gap-1 text-[9px] font-bold bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 px-2 py-0.5 rounded-md border border-orange-200 dark:border-orange-800">
                <Sliders size={9} /> {t("payroll.override", "OVERRIDE")}
                {overrideRate !== null && ` ${overrideRate.toString()}%`}
                {overrideAmount !== null && ` ₫${overrideAmount.toLocaleString()}`}
              </span>
            )}
          </div>
        </div>
      </div>

      {deletingId === a.id ? (
        <div className="flex items-center gap-2 animate-in fade-in duration-200">
          <Button className="px-3 py-1.5 bg-rose-600 text-white rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-rose-700 transition-all disabled:opacity-50"
            disabled={isDeleting}
            onClick={() => { handleDelete(a.id); }}
            variant="destructive"
          >
            {isDeleting ? "…" : t("common.confirm", "Confirm")}
          </Button>
          <Button className="px-3 py-1.5 bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-slate-300 dark:hover:bg-slate-600 transition-all"
            onClick={() => { setDeletingId(null); }}
            variant="ghost"
          >
            {t("common.cancel", "Cancel")}
          </Button>
        </div>
      ) : (
        <Button aria-label={t("payroll.delete_assignment", "Delete assignment")}
          className="p-2.5 text-slate-300 dark:text-slate-600 hover:text-rose-500 dark:hover:text-rose-400 rounded-xl hover:bg-white dark:hover:bg-slate-900 transition-all opacity-0 group-hover:opacity-100 active:scale-90 shadow-none hover:shadow-md border border-transparent hover:border-slate-100 dark:hover:border-slate-800"
          onClick={() => { setDeletingId(a.id); }}
          variant="ghost"
        >
          <Trash2 size={16} />
        </Button>
      )}
    </div>
  );
}
