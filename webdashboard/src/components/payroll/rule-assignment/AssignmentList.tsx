import React, { useState } from "react";

import { Users, Building, Globe, Trash2, Calendar, Sliders } from "lucide-react";
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
  grouped: {
    workspace: PayrollRuleAssignment[];
    department: PayrollRuleAssignment[];
    employee: PayrollRuleAssignment[];
  };
  deptMap: Record<string, Department>;
  onDelete: (id: string) => void;
  isDeleting: boolean;
}

export function AssignmentList({ grouped, deptMap, onDelete, isDeleting }: Props) {
  const { t } = useTranslation();
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const getAssigneeName = (a: PayrollRuleAssignment) => {
    if (a.assignee_type === "workspace")
      return t("payroll_rules.scope_workspace", "Entire Workspace");
    if (a.assignee_type === "department")
      return deptMap[a.assignee_id!]?.name || a.assignee_id?.slice(0, 8) + "…";
    return a.assignee_id?.slice(0, 8) + "…";
  };

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
          <div key={scope} className="space-y-2">
            <div className="flex items-center gap-2 px-1">
              <Icon size={12} className={colors.text + " opacity-60"} />
              <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
                {t(`payroll.scope_level_${scope}`, `${scope} level`)} ({items.length})
              </span>
            </div>
            {items.map((a) => (
              <div
                key={a.id}
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
                        <Calendar size={10} className="opacity-50" />
                        {formatDate(a.effective_from)}
                        {a.effective_to
                          ? ` → ${formatDate(a.effective_to)}`
                          : " → ∞"}
                      </span>
                      {a.override_config && (
                        <span className="flex items-center gap-1 text-[9px] font-bold bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 px-2 py-0.5 rounded-md border border-orange-200 dark:border-orange-800">
                          <Sliders size={9} /> {t("payroll.override", "OVERRIDE")}
                          {a.override_config.rate != null &&
                            ` ${a.override_config.rate}%`}
                          {a.override_config.amount != null &&
                            ` ₫${Number(
                              a.override_config.amount
                            ).toLocaleString()}`}
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                {/* Delete button */}
                {deletingId === a.id ? (
                  <div className="flex items-center gap-2 animate-in fade-in duration-200">
                    <button
                      onClick={() => handleDelete(a.id)}
                      disabled={isDeleting}
                      className="px-3 py-1.5 bg-rose-600 text-white rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-rose-700 transition-all disabled:opacity-50"
                    >
                      {isDeleting ? "…" : t("common.confirm", "Confirm")}
                    </button>
                    <button
                      onClick={() => setDeletingId(null)}
                      className="px-3 py-1.5 bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-slate-300 dark:hover:bg-slate-600 transition-all"
                    >
                      {t("common.cancel", "Cancel")}
                    </button>
                  </div>
                ) : (
                  <button
                    onClick={() => setDeletingId(a.id)}
                    className="p-2.5 text-slate-300 dark:text-slate-600 hover:text-rose-500 dark:hover:text-rose-400 rounded-xl hover:bg-white dark:hover:bg-slate-900 transition-all opacity-0 group-hover:opacity-100 active:scale-90 shadow-none hover:shadow-md border border-transparent hover:border-slate-100 dark:hover:border-slate-800"
                    aria-label={t("payroll.delete_assignment", "Delete assignment")}
                  >
                    <Trash2 size={16} />
                  </button>
                )}
              </div>
            ))}
          </div>
        );
      })}
    </div>
  );
}
