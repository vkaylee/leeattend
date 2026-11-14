import type { TFunction } from "i18next";

import { Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { PayrollRuleDefinition } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { PayrollFormulaHistory } from "./payroll-formula-history";
import RuleAssignmentManager from "./rule-assignment-manager";
import RuleTierManager from "./rule-tier-manager";

// eslint-disable-next-line complexity, react-refresh/only-export-components -- subtitle builder with 4 category branches + 3 calculation-method branches; exported for reuse
export function getRuleSubtitle(rule: PayrollRuleDefinition, t: TFunction): string {
  const categoryLabel = rule.category === "statutory_deduction"
    ? t("payroll.statutory_rules.categories.statutory_deduction", "Statutory Deduction")
    : rule.category === "allowance"
    ? t("payroll.statutory_rules.categories.allowance", "Allowance")
    : rule.category === "bonus"
    ? t("payroll.statutory_rules.categories.bonus", "Bonus")
    : t("payroll.statutory_rules.categories.deduction", "Deduction");

  const methodLabel = rule.calculation_method === "percentage"
    ? t("payroll.statutory_rules.methods.percentage", "Percentage")
    : rule.calculation_method === "fixed"
    ? t("payroll.statutory_rules.methods.fixed", "Fixed")
    : t("payroll.statutory_rules.methods.tiered", "Tiered");

  const parts: string[] = [categoryLabel, methodLabel];
  if (rule.calculation_method === "percentage" && rule.rule_config && typeof rule.rule_config === "object") {
    const config = rule.rule_config as { rate?: number; cap_amount?: number; base?: string };
    if (config.rate !== undefined) {
      let details = `${config.rate}%`;
      if (config.base) details += ` on ${config.base}`;
      if (config.cap_amount != null) {
        details += ` • Cap: ${config.cap_amount.toLocaleString()} ₫`;
      }
      parts.push(`(${details})`);
    }
  } else if (rule.calculation_method === "fixed" && rule.rule_config && typeof rule.rule_config === "object") {
    const config = rule.rule_config as { amount?: number };
    if (config.amount !== undefined) parts.push(`(${config.amount.toLocaleString()} ₫)`);
  } else if (rule.calculation_method === "tiered" && rule.rule_config && typeof rule.rule_config === "object") {
    const config = rule.rule_config as { base?: string };
    if (config.base) parts.push(`(on ${config.base})`);
  }
  return parts.join(" • ");
}

export function RuleStatusBadge({ isMandatory }: { isMandatory: boolean }) {
  const { t } = useTranslation();
  return (
    <span className={cn("text-[9px] font-bold uppercase tracking-widest px-3 py-1 rounded-full border shadow-sm", isMandatory ? 'bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>
      {isMandatory ? t("payroll.statutory_rules.mandatory") : t("payroll.statutory_rules.optional")}
    </span>
  );
}

interface RuleRowProps {
  expandedRuleId: string | null;
  rule: PayrollRuleDefinition;
  setExpandedRuleId: (id: string | null) => void;
}

export function RuleRow({ rule, expandedRuleId, setExpandedRuleId}: RuleRowProps) {
  const { t } = useTranslation();
  const isExpanded = expandedRuleId === rule.id;
  const toggleExpand = () => { setExpandedRuleId(isExpanded ? null : rule.id); };
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleExpand(); }
  };

  const appliesToLabel = rule.applies_to === "employee"
    ? t("payroll.statutory_rules.applies.employee")
    : t("payroll.statutory_rules.applies.employer");

  return (
    <div className="border-b border-slate-50 dark:border-slate-800/50 last:border-0">
      <div
        className={cn("p-8 px-10 flex items-center justify-between transition-all group relative m-2 rounded-3xl cursor-pointer", isExpanded ? "bg-slate-50 dark:bg-slate-800/80 shadow-inner" : "hover:bg-slate-50 dark:hover:bg-slate-800/50")}
        onClick={toggleExpand}
        onKeyDown={handleKeyDown}
        role="button"
        tabIndex={0}
      >
        <div className="flex items-center gap-6">
          <div className="w-14 h-14 rounded-2xl bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 font-bold text-lg shadow-sm group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 transition-all uppercase">{rule.code.slice(0, 2)}</div>
          <div>
            <Typography className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none" variant="p">{rule.name}</Typography>
            <Typography className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-2 opacity-70" variant="p">
              {getRuleSubtitle(rule, t)}
            </Typography>
          </div>
        </div>
        <div className="flex items-center gap-10">
          <div className="flex flex-col items-end gap-2">
            <RuleStatusBadge isMandatory={rule.is_mandatory} />
            <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-50 italic">{appliesToLabel}</span>
          </div>
          <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 active:scale-90" onClick={(e: React.MouseEvent) => { e.stopPropagation(); }} variant="ghost"><Trash2 size={18} /></Button>
        </div>
      </div>

      {isExpanded ? (
        <div className="px-12 pb-8 pt-2 animate-in slide-in-from-top-4 duration-300 space-y-6">
          {rule.calculation_method === "tiered" ? <RuleTierManager ruleId={rule.id}  /> : null}
          <RuleAssignmentManager calculationMethod={rule.calculation_method} ruleId={rule.id}  />
          {rule.code.startsWith("kpi.") ? <PayrollFormulaHistory ruleId={rule.id} /> : null}
        </div>
      ) : null}
    </div>
  );
}
