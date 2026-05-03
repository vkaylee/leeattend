import { Building, Globe, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

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
  },
  department: {
    bg: "bg-sky-50 dark:bg-sky-950/40",
    text: "text-sky-600 dark:text-sky-400",
    border: "border-sky-100 dark:border-sky-800",
  },
  employee: {
    bg: "bg-teal-50 dark:bg-teal-950/40",
    text: "text-teal-600 dark:text-teal-400",
    border: "border-teal-100 dark:border-teal-800",
  },
} as const;

export type ScopeType = "workspace" | "department" | "employee";

interface ScopeSelectorProps {
  assigneeType: ScopeType;
  setAssigneeId: (id: string) => void;
  setAssigneeType: (type: ScopeType) => void;
}

export function ScopeSelector({
  assigneeType,
  setAssigneeId,
  setAssigneeType,
}: ScopeSelectorProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-3">
      <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 mb-1">
        {t("payroll.assignment_scope", "Assignment Scope")}
      </div>
      <div className="grid grid-cols-3 gap-3">
        {(["employee", "department", "workspace"] as const).map((scope) => {
          const Icon = SCOPE_ICONS[scope];
          const colors = SCOPE_COLORS[scope];
          const isActive = assigneeType === scope;
          return (
            <button
              className={cn(
                "flex flex-col items-center gap-2.5 p-5 rounded-2xl border-2 transition-all active:scale-[0.97]",
                isActive
                  ? `${colors.bg} ${colors.border} shadow-lg`
                  : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:border-slate-200 dark:hover:border-slate-700 hover:bg-white dark:hover:bg-slate-900"
              )}
              key={scope}
              onClick={() => {
                setAssigneeType(scope);
                setAssigneeId("");
              }}
              type="button"
            >
              <Icon
                className={
                  isActive ? colors.text : "text-slate-400 dark:text-slate-500"
                }
                size={22}
              />
              <span
                className={cn(
                  "text-[10px] font-bold uppercase tracking-widest",
                  isActive
                    ? "text-slate-700 dark:text-slate-200"
                    : "text-slate-500 dark:text-slate-400"
                )}
              >
                {t(`payroll.scope_${scope}`, scope)}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
