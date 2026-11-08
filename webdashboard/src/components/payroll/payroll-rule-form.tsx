import { useMutation } from "@tanstack/react-query";
import { CheckCircle2, Calculator, Percent } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { CreatePayrollRuleDefinitionRequest, PayrollRuleDefinition, SalaryBaseGroup } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { createPayrollRule } from "@/lib/api/payroll";
import { cn } from "@/lib/utils";

import { asFixedConfig, asPercentageConfig, asTieredConfig } from "./payroll-rule-config";

interface RuleBasicFieldsProps {
  formData: Partial<PayrollRuleDefinition>;
  setFormData: (data: Partial<PayrollRuleDefinition>) => void;
}

export function RuleBasicFields({ formData, setFormData }: RuleBasicFieldsProps) {
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

function PercentageConfigFields({ formData, setFormData, bases }: { formData: Partial<PayrollRuleDefinition>; setFormData: (data: Partial<PayrollRuleDefinition>) => void; bases: SalaryBaseGroup[]; }) {
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
            {bases.map(b => (<option key={b.slug} value={b.slug}>{b.name}</option>))}
            {bases.length === 0 ? (<><option value="gross">Gross Salary (Total)</option><option value="taxable_income">Taxable Income</option></>) : null}
          </select>
        </div>
      </div>
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-cap">{t("payroll.statutory_rules.cap_amount")}</label>
        <input className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm" id="rule-cap" onChange={e => {
          const val = e.target.value ? parseFloat(e.target.value) : undefined;
          setFormData({...formData, rule_config: { ...config, cap_amount: val }});
        }} placeholder={t("payroll.statutory_rules.cap_amount_placeholder")} type="number" value={config.cap_amount ?? ""} />
        <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-2 ml-1 italic" variant="p">{t("payroll.statutory_rules.cap_amount_guide")}</Typography>
      </div>
    </div>
  );
}

function FixedConfigFields({ formData, setFormData }: { formData: Partial<PayrollRuleDefinition>; setFormData: (data: Partial<PayrollRuleDefinition>) => void; }) {
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

function TieredConfigFields({ formData, setFormData, bases }: { formData: Partial<PayrollRuleDefinition>; setFormData: (data: Partial<PayrollRuleDefinition>) => void; bases: SalaryBaseGroup[]; }) {
  const { t } = useTranslation();
  const config = asTieredConfig(formData.rule_config);
  return (
    <div className="grid grid-cols-1 gap-8">
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="rule-tier-base">{t("payroll.statutory_rules.tier_base_amount")}</label>
        <select className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm uppercase tracking-tight" id="rule-tier-base" onChange={e => { setFormData({...formData, rule_config: {...config, base: e.target.value}}); }} required value={config.base}>
          {bases.map(b => (<option key={b.slug} value={b.slug}>{b.name}</option>))}
          {bases.length === 0 ? (<><option value="taxable_income">Taxable Income</option><option value="gross">Gross Salary (Total)</option></>) : null}
        </select>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-2 ml-1 italic" variant="p">{t("payroll.statutory_rules.tier_bracket_note")}</Typography>
      </div>
    </div>
  );
}

function RuleLogicConfig({ formData, setFormData, bases }: {
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
      {formData.calculation_method === "percentage" ? <PercentageConfigFields bases={bases} formData={formData} setFormData={setFormData} /> : null}
      {formData.calculation_method === "fixed" ? <FixedConfigFields formData={formData} setFormData={setFormData} /> : null}
      {formData.calculation_method === "tiered" ? <TieredConfigFields bases={bases} formData={formData} setFormData={setFormData} /> : null}
    </div>
  );
}

interface PayrollRuleFormProps {
  bases: SalaryBaseGroup[];
  onCancel: () => void;
  onSuccess: () => void;
}

export function PayrollRuleForm({bases, onCancel, onSuccess }: PayrollRuleFormProps) {
  const { t } = useTranslation();
  const [formData, setFormData] = useState<Partial<PayrollRuleDefinition>>({ name: "", description: "", code: "", category: "statutory_deduction", applies_to: "employee", calculation_method: "percentage", is_mandatory: true, display_order: 1, rule_config: { rate: 0, base: "gross" } });

  const createMutation = useMutation({
    mutationFn: (data: CreatePayrollRuleDefinitionRequest) => createPayrollRule(data),
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
        {[{ key: "is_mandatory", label: t("payroll.statutory_rules.mandatory_rule"), desc: t("payroll.statutory_rules.mandatory_rule_desc"), icon: CheckCircle2 }].map(field => (
          <React.Fragment key={field.key}>
            <label className={cn("flex items-center gap-6 p-8 rounded-[2rem] cursor-pointer transition-all group/check border-2 shadow-sm active:scale-[0.98]",
              (formData as Record<string, boolean>)[field.key] ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800 shadow-xl shadow-primary-900/5" : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:bg-white dark:hover:bg-slate-900 hover:border-slate-100 dark:border-slate-800 shadow-inner"
            )} htmlFor={`rule-${field.key}`}>
              <input aria-label={field.label} checked={(formData as Record<string, boolean>)[field.key]} className="w-7 h-7 rounded-xl text-primary-600 dark:text-primary-400 focus:ring-primary-500 transition-all border-slate-200 dark:border-slate-700 shadow-inner cursor-pointer" id={`rule-${field.key}`} onChange={e => { setFormData({...formData, [field.key]: e.target.checked}); }} type="checkbox" />
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
