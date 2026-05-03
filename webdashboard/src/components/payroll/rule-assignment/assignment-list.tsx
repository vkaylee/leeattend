import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { PayrollRuleAssignment, Department } from "@/lib/api/types";

import { AssignmentCard, SCOPE_COLORS, SCOPE_ICONS } from "./assignment-card";

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
                t={t}
              />
            ))}
          </div>
        );
      })}
    </div>
  );
}
