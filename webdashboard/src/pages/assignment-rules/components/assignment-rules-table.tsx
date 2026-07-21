import { Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { AssignmentRule } from "@/lib/api";

import { AssignmentRuleRow } from "./assignment-rule-row";

interface AssignmentRulesTableProps {
  departments: { id: string; name: string }[] | undefined;
  employers: { id: string; name: string }[] | undefined;
  isLoading: boolean;
  locations: { id: string; name: string }[] | undefined;
  onDelete: (id: string) => void;
  onEdit: (rule: AssignmentRule) => void;
  patterns: { id: string; name: string }[] | undefined;
  rules: AssignmentRule[] | undefined;
}

export const AssignmentRulesTable: React.FC<AssignmentRulesTableProps> = ({
  departments,
  employers,
  locations,
  isLoading,
  onDelete,
  onEdit,
  patterns,
  rules,
}) => {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl sm:rounded-[3rem] overflow-hidden shadow-sm dark:shadow-black/10 animate-in slide-in-from-bottom-8 duration-700">
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
            <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.priority")}</th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.name")}</th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.pattern")}</th>
            <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.conditions")}</th>
            <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {Array.isArray(rules) && rules.map((rule) => (
            <AssignmentRuleRow
              departments={departments}
              employers={employers}
              key={rule.id}
              locations={locations}
              onDelete={onDelete}
              onEdit={onEdit}
              patterns={patterns}
              rule={rule}
            />
          ))}
          {(!Array.isArray(rules) || rules.length === 0) && !isLoading && (
            <tr>
              <td className="px-10 py-32 text-center bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center" colSpan={5}>
                <Zap className="text-slate-200 dark:text-slate-700 mb-4" size={40} />
                <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">{t("rules.empty_list")}</Typography>
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
};
