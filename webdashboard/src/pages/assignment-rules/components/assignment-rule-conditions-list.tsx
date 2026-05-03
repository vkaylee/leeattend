import { Building2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { AssignmentRuleConditions } from "@/lib/api";

interface AssignmentRuleConditionsListProps {
  conditions: AssignmentRuleConditions;
  departments: { id: string; name: string }[] | undefined;
}

export const AssignmentRuleConditionsList: React.FC<AssignmentRuleConditionsListProps> = ({
  conditions,
  departments,
}) => {
  const { t } = useTranslation();

  if (!conditions.department_id && (!conditions.tags || conditions.tags.length === 0)) {
    return (
      <span className="text-slate-300 dark:text-slate-600 text-[10px] font-bold uppercase tracking-[0.2em] italic opacity-60">
        {t("rules.match_all")}
      </span>
    );
  }

  return (
    <div className="flex flex-wrap gap-2.5">
      {!!conditions.department_id && (
        <span className="bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800 flex items-center gap-2 shadow-sm dark:shadow-black/10">
          <Building2 className="opacity-50" size={12} /> {departments?.find(d => d.id === conditions.department_id)?.name.toUpperCase()}
        </span>
      )}
      {conditions.tags?.map(tag => (
        <span className="bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm dark:shadow-black/10" key={tag}>
          #{tag.toUpperCase()}
        </span>
      ))}
    </div>
  );
};
