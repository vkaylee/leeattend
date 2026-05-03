import { Users, Building, Globe, Trash2, Calendar, Sliders } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { PayrollRuleAssignment, Department } from "@/lib/api/types";
import { cn } from "@/lib/utils";

const SCOPE_ICONS = {
  workspace: Globe,
  department: Building,
  employee: Users,
} as const;

const SCOPE_COLORS = {
  workspace: {
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

function formatDate(d: string) {
  return new Date(d).toLocaleDateString("vi-VN", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

interface Props {
  deptMap: Record<string, Department>;
  grouped: {
    workspace: PayrollRuleAssignment[];
    department: PayrollRuleAssignment[];
    employee: PayrollRuleAssignment[];
  };
  isDeleting: boolean;
  onDelete: (id: string) => void;
}

function AssignmentCard({ 
  a, 
  deptMap, 
  t, 
  scope, 
  deletingId, 
  isDeleting, 
  setDeletingId, 
  handleDelete 
}: { 
  a: PayrollRuleAssignment;
  deletingId: string | null;
  deptMap: Record<string, Department>;
  handleDelete: (id: string) => void;
  isDeleting: boolean;
  scope: "workspace" | "department" | "employee";
  setDeletingId: (id: string | null) => void;
  t: (k: string, d?: string) => string;
}) {
  const colors = SCOPE_COLORS[scope];
  const Icon = SCOPE_ICONS[scope];

  const getAssigneeName = (assignment: PayrollRuleAssignment) => {
    if (assignment.assignee_type === "workspace") {
      return t("payroll_rules.scope_workspace", "Entire Workspace");
    }
    if (assignment.assignee_type === "department") {
      const deptName = assignment.assignee_id ? deptMap[assignment.assignee_id]?.name : null;
      return deptName || (assignment.assignee_id ? `${assignment.assignee_id.slice(0, 8)}…` : "Unknown");
    }
    return assignment.assignee_id ? `${assignment.assignee_id.slice(0, 8)}…` : "Unknown";
  };

  const overrideRate = a.override_config && 'rate' in a.override_config && a.override_config.rate != null ? Number(a.override_config.rate) : null;
  const overrideAmount = a.override_config && 'amount' in a.override_config && a.override_config.amount != null ? Number(a.override_config.amount) : null;

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
          <p className="font-bold text-sm text-slate-700 dark:text-slate-200 uppercase tracking-tight truncate leading-none">
            {getAssigneeName(a)}
          </p>
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
          <button
            className="px-3 py-1.5 bg-rose-600 text-white rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-rose-700 transition-all disabled:opacity-50"
            disabled={isDeleting}
            onClick={() => { handleDelete(a.id); }}
          >
            {isDeleting ? "…" : t("common.confirm", "Confirm")}
          </button>
          <button
            className="px-3 py-1.5 bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-slate-300 dark:hover:bg-slate-600 transition-all"
            onClick={() => { setDeletingId(null); }}
          >
            {t("common.cancel", "Cancel")}
          </button>
        </div>
      ) : (
        <button
          aria-label={t("payroll.delete_assignment", "Delete assignment")}
          className="p-2.5 text-slate-300 dark:text-slate-600 hover:text-rose-500 dark:hover:text-rose-400 rounded-xl hover:bg-white dark:hover:bg-slate-900 transition-all opacity-0 group-hover:opacity-100 active:scale-90 shadow-none hover:shadow-md border border-transparent hover:border-slate-100 dark:hover:border-slate-800"
          onClick={() => { setDeletingId(a.id); }}
        >
          <Trash2 size={16} />
        </button>
      )}
    </div>
  );
}

export function AssignmentList({ grouped, deptMap, onDelete, isDeleting }: Props) {
  const { t } = useTranslation();
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const handleDelete = (id: string) => {
    onDelete(id);
    setDeletingId(null);
  };

  return (
    <div className="space-y-3">
      {(["workspace", "department", "employee"] as const).map((scope) => {
        const items = grouped[scope];
        if (items.length === 0) return null;
        const colors = SCOPE_COLORS[scope];
        const Icon = SCOPE_ICONS[scope];
        return (
          <div className="space-y-2" key={scope}>
            <div className="flex items-center gap-2 px-1">
              <Icon className={colors.text + " opacity-60"} size={12} />
              <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                {t(`payroll.scope_level_${scope}`, `${scope} level`)} ({items.length})
              </span>
            </div>
            {items.map((a) => (
              <AssignmentCard
                a={a}
                deletingId={deletingId}
                deptMap={deptMap}
                handleDelete={handleDelete}
                isDeleting={isDeleting}
                key={a.id}
                scope={scope}
                setDeletingId={setDeletingId}
                t={t as any}
              />
            ))}
          </div>
        );
      })}
    </div>
  );
}
