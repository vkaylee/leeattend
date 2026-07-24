import { Building2, MapPin, Briefcase, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { AssignmentRuleConditions } from "@/lib/api";

interface AssignmentRuleConditionsListProps {
  conditions: AssignmentRuleConditions;
  departments: { id: string; name: string }[] | undefined;
  employers?: { id: string; name: string }[] | undefined;
  locations?: { id: string; name: string }[] | undefined;
}

export const AssignmentRuleConditionsList: React.FC<AssignmentRuleConditionsListProps> = ({
  conditions,
  departments,
  employers,
  locations,
  // eslint-disable-next-line complexity
}) => {
  const { t } = useTranslation();

  /* eslint-disable @typescript-eslint/prefer-nullish-coalescing -- || is used for boolean OR between different conditions, not default-coalescing */
  const hasConditions =
    (conditions.department_id ?? conditions.employer_id ?? conditions.location_id) ||
    (conditions.tags && conditions.tags.length > 0) ||
    (conditions.employee_id_numbers && conditions.employee_id_numbers.length > 0);
  /* eslint-enable @typescript-eslint/prefer-nullish-coalescing */

  if (!hasConditions) {
    return (
      <span className="text-slate-300 dark:text-slate-600 text-[10px] font-bold uppercase tracking-[0.2em] italic opacity-60">
        {t("rules.match_all", "Match All Employees")}
      </span>
    );
  }

  return (
    <div className="flex flex-wrap gap-2.5">
      {/* Employer */}
      {!!conditions.employer_id && (
        <span className="bg-purple-50 dark:bg-purple-950 text-purple-600 dark:text-purple-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-purple-100 dark:border-purple-800 flex items-center gap-2 shadow-sm dark:shadow-black/10">
          <Briefcase className="opacity-50" size={12} /> {employers?.find(e => e.id === conditions.employer_id)?.name.toUpperCase() ?? "EMPLOYER"}
        </span>
      )}

      {/* Location */}
      {!!conditions.location_id && (
        <span className="bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-rose-100 dark:border-rose-800 flex items-center gap-2 shadow-sm dark:shadow-black/10">
          <MapPin className="opacity-50" size={12} /> {locations?.find(l => l.id === conditions.location_id)?.name.toUpperCase() ?? "LOCATION"}
        </span>
      )}

      {/* Department */}
      {!!conditions.department_id && (
        <span className="bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800 flex items-center gap-2 shadow-sm dark:shadow-black/10">
          <Building2 className="opacity-50" size={12} /> {departments?.find(d => d.id === conditions.department_id)?.name.toUpperCase()}
        </span>
      )}

      {/* Tags */}
      {conditions.tags?.map(tag => (
        <span className="bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm dark:shadow-black/10" key={tag}>
          #{tag.toUpperCase()}
        </span>
      ))}

      {/* Specific Employee IDs */}
      {!!conditions.employee_id_numbers && conditions.employee_id_numbers.length > 0 && (
        <span className="bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-indigo-100 dark:border-indigo-800 flex items-center gap-2 shadow-sm dark:shadow-black/10" title={conditions.employee_id_numbers.join(", ")}>
          <Users className="opacity-50" size={12} /> {conditions.employee_id_numbers.length} EMPLOYEES
        </span>
      )}
    </div>
  );
};
