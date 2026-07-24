import { Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import StatusBadge from "@/components/ui/status-badge";
import { AssignmentRule } from "@/lib/api";

import { AssignmentRuleConditionsList } from "./assignment-rule-conditions-list";

interface AssignmentRuleRowProps {
  departments: { id: string; name: string }[] | undefined;
  employers?: { id: string; name: string }[] | undefined;
  locations?: { id: string; name: string }[] | undefined;
  onDelete: (id: string) => void;
  onEdit: (rule: AssignmentRule) => void;
  patterns: { id: string; name: string }[] | undefined;
  rule: AssignmentRule;
}

export const AssignmentRuleRow: React.FC<AssignmentRuleRowProps> = ({
  departments,
  employers,
  locations,
  onDelete,
  onEdit,
  patterns,
  rule,
}) => {
  const { t } = useTranslation();

  return (
    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group" key={rule.id}>
      <td className="px-10 py-6">
        <span className="bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 px-3 py-1.5 rounded-xl text-[10px] font-bold tracking-tight border border-slate-200 dark:border-slate-700 shadow-inner font-mono uppercase">
          P{rule.priority}
        </span>
      </td>
      <td className="px-8 py-6">
        <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1">{rule.name}</div>
        <div className="text-[9px] text-slate-400 dark:text-slate-500 font-semibold tracking-tight mb-2">
          {rule.start_date ?? t("rules.start_date_fallback", "Từ trước")} ~ {rule.end_date ?? t("rules.end_date_fallback", "Vô thời hạn")}
        </div>
        {!rule.is_active && <StatusBadge variant="danger">{t("common.inactive").toUpperCase()}</StatusBadge>}
      </td>
      <td className="px-8 py-6">
        <span className="bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 px-3 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-primary-100 dark:border-primary-800 shadow-sm dark:shadow-black/10">
          {patterns?.find(p => p.id === rule.pattern_id)?.name.toUpperCase() ?? "UNKNOWN"}
        </span>
      </td>
      <td className="px-8 py-6">
        <AssignmentRuleConditionsList
          conditions={rule.conditions}
          departments={departments}
          employers={employers}
          locations={locations}
        />
      </td>
      <td className="px-10 py-6 text-right">
        <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
          <Button className="p-3 bg-white dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 rounded-xl shadow-md dark:shadow-black/20 border border-slate-100 dark:border-slate-700 transition-all active:scale-90" 
            onClick={() => { onEdit(rule); }} 
            title={t("common.edit")} 
            variant="ghost"
          >
            <Pencil size={18} />
          </Button>
          <Button className="p-3 bg-white dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 rounded-xl shadow-md dark:shadow-black/20 border border-slate-100 dark:border-slate-700 transition-all active:scale-90" 
            onClick={() => { onDelete(rule.id); }} 
            title={t("common.delete")} 
            variant="ghost"
          >
            <Trash2 size={18} />
          </Button>
        </div>
      </td>
    </tr>
  );
};
