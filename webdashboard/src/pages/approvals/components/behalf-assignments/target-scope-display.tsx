import React from "react";
import { useTranslation } from "react-i18next";

import type { TargetScopeDisplayProps } from "./types";

export function TargetScopeDisplay({ targetEmp, targetDept }: TargetScopeDisplayProps) {
  const { t } = useTranslation();

  if (targetEmp) {
    return (
      <div className="mt-0.5">
        <span className="inline-flex items-center px-2 py-0.5 rounded-lg text-xs font-semibold bg-indigo-50 dark:bg-indigo-950/40 text-indigo-700 dark:text-indigo-400 mr-2">
          {t("behalf_assignments.scope_type_employee")}
        </span>
        <span className="font-bold text-slate-700 dark:text-slate-300 text-sm">
          {targetEmp.full_name}
        </span>
        {targetEmp.employee_id_number ? (
          <span className="text-[10px] text-slate-400 font-mono ml-2">
            ({targetEmp.employee_id_number})
          </span>
        ) : null}
      </div>
    );
  }

  if (targetDept) {
    return (
      <div className="mt-0.5">
        <span className="inline-flex items-center px-2 py-0.5 rounded-lg text-xs font-semibold bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-indigo-400 mr-2">
          {t("behalf_assignments.scope_type_department")}
        </span>
        <span className="font-bold text-slate-700 dark:text-slate-300 text-sm">
          {targetDept.name}
        </span>
        {targetDept.description ? (
          <span className="text-[10px] text-slate-400 ml-2">
            ({targetDept.description})
          </span>
        ) : null}
      </div>
    );
  }

  return <span className="text-slate-400 text-sm mt-0.5 italic">{t("behalf_assignments.scope_type_none")}</span>;
}
