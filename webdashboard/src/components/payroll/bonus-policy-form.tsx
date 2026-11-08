import { Plus, Save, TrendingUp, Trash2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useBonusTiers, type TierInput } from "@/hooks/use-bonus-tiers";
import { createIncentivePolicy } from "@/lib/api/payroll";
import { RuleCondition } from "@/lib/api/types";

import { METRICS, OPERATORS } from "./bonus-policy-constants";

// ── Bonus Tier Input ─────────────────────────────────────────────────

interface BonusTierInputProps {
  addTierRule: (tierIndex: number) => void;
  handleRemoveTier: (index: number) => void;
  index: number;
  removeTierRule: (tierIndex: number, ruleIndex: number) => void;
  tier: TierInput;
  updateTierAmount: (index: number, value: string | number) => void;
  updateTierRule: (tierIndex: number, ruleIndex: number, field: keyof RuleCondition, value: string | number) => void;
}

function BonusTierInput({
  tier, index, updateTierAmount, updateTierRule, addTierRule, removeTierRule, handleRemoveTier,
}: BonusTierInputProps) {
  return (
    <div className="bg-white dark:bg-slate-900 p-8 rounded-3xl border border-slate-100 dark:border-slate-800 shadow-sm flex flex-col items-stretch gap-8 group relative">
      <div className="flex items-center justify-between">
        <div className="w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-700 flex items-center justify-center font-bold text-slate-500 dark:text-slate-400 text-xs shadow-inner">#{index + 1}</div>
        <div className="flex items-center gap-2">
          <div className="space-y-1 w-40">
            <label className="text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest flex items-center gap-1.5" htmlFor={`payout-${index}`}><TrendingUp size={10} /> Payout Amount</label>
            <input
              className="w-full bg-primary-50 dark:bg-primary-950/50 border border-primary-100 dark:border-primary-800 rounded-xl px-4 py-2 outline-none focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-xs font-mono text-primary-700 dark:text-primary-300"
              id={`payout-${index}`}
              onChange={e => { updateTierAmount(index, e.target.value); }}
              type="number"
              value={tier.payout_amount}
            />
          </div>
          <Button className="w-10 h-10 rounded-xl text-slate-300 dark:text-slate-600 hover:text-rose-500 hover:bg-rose-50 dark:bg-rose-950 transition-all flex items-center justify-center mt-4"
            onClick={() => { handleRemoveTier(index); }}
            variant="ghost"
          >
            <Trash2 size={18} />
          </Button>
        </div>
      </div>

      <div className="space-y-4 bg-slate-50 dark:bg-slate-800/50 p-4 rounded-2xl">
        <div className="flex items-center justify-between">
          <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500">Conditions (AND)</span>
          <Button className="text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900 px-3 py-1.5 rounded-lg text-[9px] font-bold uppercase tracking-widest flex items-center gap-1" onClick={() => { addTierRule(index); }} variant="primary">
            <Plus size={10} /> Add Rule
          </Button>
        </div>
        {tier.rule_config.length === 0 ? (
          <Typography className="text-xs text-slate-400 italic" variant="p">No conditions. This tier applies to everyone.</Typography>
        ) : (
          tier.rule_config.map((rule, ruleIdx) => (
            <div className="flex items-center gap-2" key={ruleIdx}>
              <select
                className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-2 outline-none text-xs font-bold"
                onChange={e => { updateTierRule(index, ruleIdx, "metric", e.target.value); }}
                value={rule.metric}
              >
                {METRICS.map(m => <option key={m.value} value={m.value}>{m.label}</option>)}
              </select>
              <select
                className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-2 outline-none text-xs font-bold text-primary-600"
                onChange={e => { updateTierRule(index, ruleIdx, "operator", e.target.value); }}
                value={rule.operator}
              >
                {OPERATORS.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
              </select>
              <input
                className="w-24 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-2 outline-none text-xs font-bold font-mono"
                onChange={e => { updateTierRule(index, ruleIdx, "value", e.target.value === "" ? 0 : Number(e.target.value)); }}
                type="number"
                value={rule.value}
              />
              <Button className="p-2 text-slate-400 hover:text-rose-500 rounded-lg transition-colors"
                onClick={() => { removeTierRule(index, ruleIdx); }}
                variant="ghost"
              >
                <Trash2 size={14} />
              </Button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

// ── Bonus Policy Inputs ──────────────────────────────────────────────

interface BonusPolicyInputsProps {
  baseAmount: number; effectiveFrom: string;
  name: string; setBaseAmount: (val: number) => void;
  setEffectiveFrom: (val: string) => void; setName: (val: string) => void;
}

function BonusPolicyInputs({ name, setName, baseAmount, setBaseAmount, effectiveFrom, setEffectiveFrom }: BonusPolicyInputsProps) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.incentives.policy_name")}</label>
        <input className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm"
          onChange={e => { setName(e.target.value); }} placeholder="e.g. Diligence Bonus" type="text" value={name}
        />
      </div>
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.incentives.base_amount")}</label>
        <input className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm font-mono"
          onChange={e => { setBaseAmount(parseFloat(e.target.value)); }} type="number" value={baseAmount}
        />
      </div>
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.incentives.effective_from", { defaultValue: "EFFECTIVE FROM" })}</label>
        <input className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm"
          onChange={e => { setEffectiveFrom(e.target.value); }} type="date" value={effectiveFrom}
        />
      </div>
    </div>
  );
}

// ── Bonus Policy Form ────────────────────────────────────────────────

interface BonusPolicyFormProps {
  isPending: boolean;
  onCancel: () => void;
  onSave: (data: Parameters<typeof createIncentivePolicy>[0]) => void;
}

export function BonusPolicyForm({ onCancel, onSave, isPending }: BonusPolicyFormProps) {
  const { t } = useTranslation();
  const toast = useToast();

  const [name, setName] = useState("");
  const [baseAmount, setBaseAmount] = useState(300000);
  const [effectiveFrom, setEffectiveFrom] = useState(() => new Date().toISOString().split('T')[0]);
  const [payoutType] = useState<"fixed" | "percentage">("fixed");
  const { tiers, handleAddTier, handleRemoveTier, updateTierAmount, updateTierRule, addTierRule, removeTierRule } = useBonusTiers([
    { priority: 1, payout_amount: 150000, rule_config: [{ metric: "late_early_count", operator: "<=", value: 2 }] },
    { priority: 2, payout_amount: 0, rule_config: [] }
  ]);

  const handleSave = () => {
    if (!name) { toast.error(t("payroll.incentives.name_required")); return; }
    onSave({
      name,
      base_amount: baseAmount,
      payout_type: payoutType,
      effective_from: effectiveFrom,
      tiers: tiers.map(t => ({
        ...t,
        payout_amount: t.payout_amount,
        rule_config: t.rule_config
      }))
    });
  };

  return (
    <div className="bg-slate-50 dark:bg-slate-800 rounded-2xl sm:rounded-[2.5rem] p-6 sm:p-10 border border-slate-200 dark:border-slate-700 shadow-inner animate-in slide-in-from-top-4 duration-500 space-y-10">
      <BonusPolicyInputs baseAmount={baseAmount} effectiveFrom={effectiveFrom} name={name} setBaseAmount={setBaseAmount} setEffectiveFrom={setEffectiveFrom} setName={setName} />

      <div className="space-y-6">
        <div className="flex items-center justify-between px-1">
          <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" variant="h3">{t("payroll.incentives.tiers_config", { defaultValue: "EVALUATION TIERS" })}</Typography>
          <Button className="text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:text-primary-300 font-bold text-[10px] uppercase tracking-widest flex items-center gap-1.5" onClick={handleAddTier} variant="ghost"><Plus size={14} /> {t("common.add")}</Button>
        </div>

        <div className="space-y-4">
          {tiers.map((tier, index) => (
            <BonusTierInput
              addTierRule={addTierRule}
              handleRemoveTier={handleRemoveTier}
              index={index}
              key={index}
              removeTierRule={removeTierRule}
              tier={tier}
              updateTierAmount={updateTierAmount}
              updateTierRule={updateTierRule}
            />
          ))}
        </div>
      </div>

      <div className="flex justify-end gap-4 pt-4">
        <Button className="bg-transparent hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-500 dark:text-slate-400 px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] transition-all" onClick={onCancel} variant="ghost">
          {t("common.cancel")}
        </Button>
        <Button className="bg-slate-900 hover:bg-primary-700 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] shadow-2xl active:scale-95 transition-all flex items-center gap-3 disabled:opacity-50"
          disabled={isPending} onClick={handleSave} variant="ghost"
        >
          {isPending ? <Plus className="animate-spin" size={18} /> : <Save size={18} />}
          {t("common.save_settings")}
        </Button>
      </div>
    </div>
  );
}
