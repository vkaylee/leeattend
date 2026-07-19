import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, CheckCircle2, FileText, Trash2, Percent, Calculator } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchPayrollRules, createPayrollRule, fetchSalaryBaseGroups } from "@/lib/api/payroll";
import { CreatePayrollRuleDefinitionRequest, PayrollRuleDefinition, SalaryBaseGroup } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import RuleAssignmentManager from "./rule-assignment-manager";
import RuleTierManager from "./rule-tier-manager";

// ── Type-safe rule_config shapes ────────────────────────────────────────────
// The API returns rule_config as `unknown` because it varies by calculation_method.
// These types provide safe access within the editor sub-components.

interface PercentageConfig { base: string; cap_amount?: number; rate: number }
interface FixedConfig { amount: number }
interface TieredConfig { base: string }

/** Narrow the unknown rule_config to PercentageConfig, defaulting missing fields. */
function asPercentageConfig(raw: unknown): PercentageConfig {
  const c = (raw ?? {}) as Record<string, unknown>;
  return {
    rate: typeof c.rate === "number" ? c.rate : 0,
    base: typeof c.base === "string" ? c.base : "gross",
    cap_amount: typeof c.cap_amount === "number" ? c.cap_amount : undefined,
  };
}

/** Narrow the unknown rule_config to FixedConfig, defaulting missing fields. */
function asFixedConfig(raw: unknown): FixedConfig {
  const c = (raw ?? {}) as Record<string, unknown>;
  return { amount: typeof c.amount === "number" ? c.amount : 0 };
}

/** Narrow the unknown rule_config to TieredConfig, defaulting missing fields. */
function asTieredConfig(raw: unknown): TieredConfig {
  const c = (raw ?? {}) as Record<string, unknown>;
  return { base: typeof c.base === "string" ? c.base : "taxable_income" };
}

interface Props {
  wsId: string;
}


function RuleBasicFields({
  formData,
  setFormData,
}: {
  formData: Partial<PayrollRuleDefinition>;
  setFormData: (data: Partial<PayrollRuleDefinition>) => void;
}) {
  const { t } = useTranslation();
  return (
    <>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-name">{t("common.name")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all tracking-tight shadow-sm" id="rule-name" onChange={e => { setFormData({...formData, name: e.target.value}); }} placeholder="Social Insurance" required type="text" value={formData.name} />
        </div>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-code">{t("payroll.statutory_rules.code")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all tracking-tight shadow-sm" id="rule-code" onChange={e => { setFormData({...formData, code: e.target.value.toUpperCase()}); }} placeholder="BHXH" required type="text" value={formData.code} />
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-category">{t("payroll.statutory_rules.category")}</label>
          <select className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" id="rule-category" onChange={e => { setFormData({...formData, category: e.target.value as "statutory_deduction" | "allowance" | "bonus" | "deduction"}); }} required value={formData.category}>
            <option value="statutory_deduction">{t("payroll.statutory_rules.categories.statutory_deduction")}</option>
            <option value="allowance">{t("payroll.statutory_rules.categories.allowance")}</option>
            <option value="bonus">{t("payroll.statutory_rules.categories.bonus")}</option>
            <option value="deduction">{t("payroll.statutory_rules.categories.deduction")}</option>
          </select>
        </div>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-calc">{t("payroll.statutory_rules.calc_method")}</label>
          <select className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" id="rule-calc" onChange={e => {
            const method = e.target.value as "percentage" | "fixed" | "tiered";
            let newConfig = {};
            if (method === "percentage") newConfig = { rate: 0, base: "gross" };
            else if (method === "fixed") newConfig = { amount: 0, frequency: "monthly" };
            else newConfig = { base: "taxable_income" };
            setFormData({...formData, calculation_method: method, rule_config: newConfig});
          }} required value={formData.calculation_method}>
            <option value="percentage">{t("payroll.statutory_rules.methods.percentage")}</option>
            <option value="fixed">{t("payroll.statutory_rules.methods.fixed")}</option>
            <option value="tiered">{t("payroll.statutory_rules.methods.tiered")}</option>
          </select>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-applies-to">{t("payroll.statutory_rules.applies_to")}</label>
          <select className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" id="rule-applies-to" onChange={e => { setFormData({...formData, applies_to: e.target.value as "employee" | "employer"}); }} required value={formData.applies_to}>
            <option value="employee">{t("payroll.statutory_rules.applies.employee")}</option>
            <option value="employer">{t("payroll.statutory_rules.applies.employer")}</option>
          </select>
        </div>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-display-order">{t("payroll.statutory_rules.display_order")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all tracking-tight shadow-sm" id="rule-display-order" min="1" onChange={e => { setFormData({...formData, display_order: parseInt(e.target.value) || 1}); }} required type="number" value={formData.display_order ?? 1} />
        </div>
      </div>
    </>
  );
}

function PercentageConfig({ formData, setFormData, bases }: { formData: Partial<PayrollRuleDefinition>; setFormData: (data: Partial<PayrollRuleDefinition>) => void; bases: SalaryBaseGroup[]; }) {
  const { t } = useTranslation();
  const config = asPercentageConfig(formData.rule_config);
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1" htmlFor="rule-rate"><Percent size={12}/> {t("payroll.statutory_rules.rate")}</label>
          <input className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm" id="rule-rate" onChange={e => { setFormData({...formData, rule_config: {...config, rate: parseFloat(e.target.value)}}); }} required step="0.01" type="number" value={config.rate || ""} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-calc-base">{t("payroll.statutory_rules.calc_base")}</label>
          <select className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm uppercase tracking-tight" id="rule-calc-base" onChange={e => { setFormData({...formData, rule_config: {...config, base: e.target.value}}); }} required value={config.base}>
            {bases.map(b => (
              <option key={b.slug} value={b.slug}>{b.name}</option>
            ))}
            {bases.length === 0 ? (
              <>
                <option value="gross">Gross Salary (Total)</option>
                <option value="taxable_income">Taxable Income</option>
              </>
            ) : null}
          </select>
        </div>
      </div>
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-cap">{t("payroll.statutory_rules.cap_amount")}</label>
        <input className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm" id="rule-cap" onChange={e => {
          const val = e.target.value ? parseFloat(e.target.value) : undefined;
          setFormData({
            ...formData,
            rule_config: { ...config, cap_amount: val }
          });
        }} placeholder={t("payroll.statutory_rules.cap_amount_placeholder")} type="number" value={config.cap_amount ?? ""} />
        <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-2 ml-1 italic" variant="p">{t("payroll.statutory_rules.cap_amount_guide")}</Typography>
      </div>
    </div>
  );
}

function FixedConfig({ formData, setFormData }: { formData: Partial<PayrollRuleDefinition>; setFormData: (data: Partial<PayrollRuleDefinition>) => void; }) {
  const { t } = useTranslation();
  const config = asFixedConfig(formData.rule_config);
  return (
    <div className="grid grid-cols-1 gap-8">
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-fixed">{t("payroll.statutory_rules.fixed_amount")}</label>
        <input className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm" id="rule-fixed" onChange={e => { setFormData({...formData, rule_config: {...config, amount: parseFloat(e.target.value)}}); }} required type="number" value={config.amount || ""} />
      </div>
    </div>
  );
}

function TieredConfig({ formData, setFormData, bases }: { formData: Partial<PayrollRuleDefinition>; setFormData: (data: Partial<PayrollRuleDefinition>) => void; bases: SalaryBaseGroup[]; }) {
  const { t } = useTranslation();
  const config = asTieredConfig(formData.rule_config);
  return (
    <div className="grid grid-cols-1 gap-8">
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-tier-base">{t("payroll.statutory_rules.tier_base_amount")}</label>
        <select className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm uppercase tracking-tight" id="rule-tier-base" onChange={e => { setFormData({...formData, rule_config: {...config, base: e.target.value}}); }} required value={config.base}>
          {bases.map(b => (
            <option key={b.slug} value={b.slug}>{b.name}</option>
          ))}
          {bases.length === 0 ? (
            <>
              <option value="taxable_income">Taxable Income</option>
              <option value="gross">Gross Salary (Total)</option>
            </>
          ) : null}
        </select>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-2 ml-1 italic" variant="p">{t("payroll.statutory_rules.tier_bracket_note")}</Typography>
      </div>
    </div>
  );
}

function RuleLogicConfig({
  formData,
  setFormData,
  bases
}: {
  formData: Partial<PayrollRuleDefinition>;
  setFormData: (data: Partial<PayrollRuleDefinition>) => void;
  bases: SalaryBaseGroup[];
}) {
  const { t } = useTranslation();
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 p-8 rounded-[2rem] border border-slate-100 dark:border-slate-800 space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <Calculator className="text-primary-600 dark:text-primary-400" size={18} />
        <Typography className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-widest" variant="h3">{t("payroll.statutory_rules.logic_config")}</Typography>
      </div>
      
      {formData.calculation_method === "percentage" ? (
        <PercentageConfig bases={bases} formData={formData} setFormData={setFormData} />
      ) : null}

      {formData.calculation_method === "fixed" ? (
        <FixedConfig formData={formData} setFormData={setFormData} />
      ) : null}

      {formData.calculation_method === "tiered" ? (
        <TieredConfig bases={bases} formData={formData} setFormData={setFormData} />
      ) : null}
    </div>
  );
}

function PayrollRuleForm({ 
  wsId, 
  bases, 
  onCancel, 
  onSuccess 
}: { 
  bases: SalaryBaseGroup[]; 
  onCancel: () => void; 
  onSuccess: () => void; 
  wsId: string; 
}) {
  const { t } = useTranslation();
  const [formData, setFormData] = useState<Partial<PayrollRuleDefinition>>({ name: "", description: "", code: "", category: "statutory_deduction", applies_to: "employee", calculation_method: "percentage", is_mandatory: true, display_order: 1, rule_config: { rate: 0, base: "gross" } });

  const createMutation = useMutation({
    mutationFn: (data: CreatePayrollRuleDefinitionRequest) => createPayrollRule(wsId, data),
    onSuccess,
  });

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    createMutation.mutate(formData as CreatePayrollRuleDefinitionRequest);
  };

  return (
    <form className="space-y-10 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
      <RuleBasicFields formData={formData} setFormData={setFormData} />
      <RuleLogicConfig bases={bases} formData={formData} setFormData={setFormData} />

      <div className="space-y-5">
        {[
          { key: "is_mandatory", label: t("payroll.statutory_rules.mandatory_rule"), desc: t("payroll.statutory_rules.mandatory_rule_desc"), icon: CheckCircle2 }
        ].map(field => (
          <React.Fragment key={field.key}>
            {/* eslint-disable-next-line jsx-a11y/label-has-associated-control -- dynamic id */}
            <label className={cn("flex items-center gap-6 p-8 rounded-[2rem] cursor-pointer transition-all group/check border-2 shadow-sm active:scale-[0.98]", 
              (formData as Record<string, boolean>)[field.key] ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800 shadow-xl shadow-primary-900/5" : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:bg-white dark:hover:bg-slate-900 hover:border-slate-100 dark:border-slate-800 shadow-inner"
            )} htmlFor={`rule-${field.key}`}>
              <input checked={(formData as Record<string, boolean>)[field.key]} className="w-7 h-7 rounded-xl text-primary-600 dark:text-primary-400 focus:ring-primary-500 transition-all border-slate-200 dark:border-slate-700 shadow-inner cursor-pointer" id={`rule-${field.key}`} onChange={e => { setFormData({...formData, [field.key]: e.target.checked}); }} type="checkbox" />
              <div>
                <Typography className="font-bold text-slate-800 dark:text-slate-100 text-sm uppercase tracking-tight group-hover/check:text-primary-600 dark:text-primary-400 transition-colors flex items-center gap-2" variant="p"><field.icon className="opacity-50" size={14} /> {field.label}</Typography>
                <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70 leading-relaxed max-w-sm" variant="p">{field.desc}</Typography>
              </div>
            </label>
          </React.Fragment>
        ))}
      </div>

      <div className="flex gap-6 pt-4 border-t border-slate-50 dark:border-slate-800">
        <Button className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95" onClick={onCancel} type="button" variant="primary">{t("common.cancel").toUpperCase()}</Button>
        <Button className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50" disabled={createMutation.isPending} type="submit" variant="primary">{createMutation.isPending ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</Button>
      </div>
    </form>
  );
}

interface RuleRowProps {
  expandedRuleId: string | null;
  rule: PayrollRuleDefinition;
  setExpandedRuleId: (id: string | null) => void;
  wsId: string;
}

import { type TFunction } from "i18next";

  // eslint-disable-next-line complexity
function getRuleSubtitle(rule: PayrollRuleDefinition, t: TFunction): string {
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
      if (config.base) {
        details += ` on ${config.base}`;
      }
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      if (config.cap_amount !== undefined && config.cap_amount !== null) {
        details += ` • Cap: ${config.cap_amount.toLocaleString()} ₫`;
      }
      parts.push(`(${details})`);
    }
  } else if (rule.calculation_method === "fixed" && rule.rule_config && typeof rule.rule_config === "object") {
    const config = rule.rule_config as { amount?: number };
    if (config.amount !== undefined) {
      parts.push(`(${config.amount.toLocaleString()} ₫)`);
    }
  } else if (rule.calculation_method === "tiered" && rule.rule_config && typeof rule.rule_config === "object") {
    const config = rule.rule_config as { base?: string };
    if (config.base) {
      parts.push(`(on ${config.base})`);
    }
  }
  return parts.join(" • ");
}

function RuleStatusBadge({ isMandatory }: { isMandatory: boolean }) {
  const { t } = useTranslation();
  return (
    <span className={cn("text-[9px] font-bold uppercase tracking-widest px-3 py-1 rounded-full border shadow-sm", isMandatory ? 'bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>
      {isMandatory ? t("payroll.statutory_rules.mandatory") : t("payroll.statutory_rules.optional")}
    </span>
  );
}

function RuleRow({ rule, expandedRuleId, setExpandedRuleId, wsId }: RuleRowProps) {
  const { t } = useTranslation();
  const isExpanded = expandedRuleId === rule.id;
  const toggleExpand = () => {
    setExpandedRuleId(isExpanded ? null : rule.id);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      toggleExpand();
    }
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

      {/* Expanded details section */}
      {isExpanded ? (
        <div className="px-12 pb-8 pt-2 animate-in slide-in-from-top-4 duration-300 space-y-6">
          {rule.calculation_method === "tiered" ? (
            <RuleTierManager ruleId={rule.id} wsId={wsId} />
          ) : null}
          <RuleAssignmentManager calculationMethod={rule.calculation_method} ruleId={rule.id} wsId={wsId} />
        </div>
      ) : null}
    </div>
  );
}

export default function PayrollRulesManager({ wsId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [expandedRuleId, setExpandedRuleId] = useState<string | null>(null);

  const { data: rules, isLoading } = useQuery({ queryKey: ["payroll-rules", wsId], queryFn: () => fetchPayrollRules(wsId), enabled: !!wsId });
  const { data: basesResponse } = useQuery({ queryKey: ["salary-base-groups", wsId], queryFn: () => fetchSalaryBaseGroups(wsId), enabled: !!wsId });
  const bases = basesResponse?.groups;

  return (
    <div className="animate-in fade-in duration-700 space-y-6">
      <div className="flex items-center justify-between pb-6 border-b border-slate-100 dark:border-slate-800">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-rose-50 dark:bg-rose-950 flex items-center justify-center text-rose-600 dark:text-rose-300 shadow-inner border border-rose-100 dark:border-rose-800/50"><FileText size={24} /></div>
          <div>
            <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-none" variant="h2">{t("payroll.statutory_rules.title")}</Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70" variant="p">{t("payroll.statutory_rules.subtitle")}</Typography>
          </div>
        </div>
        <Button className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" onClick={() => { setIsModalOpen(true); }} variant="ghost"><Plus size={18} /> {t("common.add").toUpperCase()}</Button>
      </div>

      <div className="divide-y divide-slate-50 dark:divide-slate-800 px-2">
        {isLoading ? (
          <div className="p-20 text-center" data-testid="loader"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : rules?.length === 0 ? (
          <div className="p-32 text-center bg-slate-50 dark:bg-slate-800/30 m-8 rounded-[3rem] border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><FileText className="text-slate-200 dark:text-slate-700" size={32} /></div>
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">{t("payroll.statutory_rules.no_rules")}</Typography>
          </div>
        ) : (
          rules?.map((rule) => (
            <RuleRow
              expandedRuleId={expandedRuleId}
              key={rule.id}
              rule={rule}
              setExpandedRuleId={setExpandedRuleId}
              wsId={wsId}
            />
          ))
        )}
      </div>

      <Modal icon={<FileText size={24} />} isOpen={isModalOpen} maxWidth="lg" onClose={() => { setIsModalOpen(false); }} subtitle={t("payroll.statutory_rules.create_rule_desc")} title={t("payroll.statutory_rules.add_rule")}>
        <PayrollRuleForm 
          bases={bases ?? []} 
          onCancel={() => { setIsModalOpen(false); }} 
          onSuccess={() => {
            toast.success(t("common.success"));
            void queryClient.invalidateQueries({ queryKey: ["payroll-rules"] });
            setIsModalOpen(false);
          }} 
          wsId={wsId} 
        />
      </Modal>
    </div>
  );
}
