import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Sparkles, 
  Plus, 
  Trash2, 
  Save,
  CheckCircle2,
  TrendingUp,
  Target
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchIncentivePolicies, createIncentivePolicy, deleteIncentivePolicy, cloneIncentivePolicy } from "@/lib/api/payroll";
import { IncentiveTier, PayrollIncentivePolicyResponse, RuleCondition } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { useBonusTiers, type TierInput } from "@/hooks/use-bonus-tiers";

interface BonusPolicyManagerProps {
  wsId: string;
}



const METRICS = [
  { value: "kpi_score", label: "KPI Score" },
  { value: "late_early_count", label: "Late/Early Count" },
  { value: "total_work_units", label: "Total Work Units" },
  { value: "total_leave_days", label: "Total Leave Days" },
  { value: "unpaid_leave_days", label: "Unpaid Leave Days" },
  { value: "sick_leave_days", label: "Sick Leave Days" }
];

const OPERATORS = [
  { value: ">=", label: "≥ Greater or Equal" },
  { value: "<=", label: "≤ Less or Equal" },
  { value: "==", label: "= Equals" },
  { value: ">", label: "> Greater Than" },
  { value: "<", label: "< Less Than" }
];

function BonusTierInput({
  tier,
  index,
  updateTierAmount,
  updateTierRule,
  addTierRule,
  removeTierRule,
  handleRemoveTier
}: {
  tier: TierInput;
  index: number;
  updateTierAmount: (index: number, value: string | number) => void;
  updateTierRule: (tierIndex: number, ruleIndex: number, field: keyof RuleCondition, value: string | number) => void;
  addTierRule: (tierIndex: number) => void;
  removeTierRule: (tierIndex: number, ruleIndex: number) => void;
  handleRemoveTier: (index: number) => void;
}) {
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

function BonusPolicyInputs({
  name, setName,
  baseAmount, setBaseAmount,
  effectiveFrom, setEffectiveFrom,
}: {
  baseAmount: number; setBaseAmount: (val: number) => void;
  effectiveFrom: string; setEffectiveFrom: (val: string) => void;
  name: string; setName: (val: string) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.incentives.policy_name")}</label>
        <input 
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm" 
          onChange={e => { setName(e.target.value); }} 
          placeholder="e.g. Diligence Bonus"
          type="text"
          value={name}
        />
      </div>
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.incentives.base_amount")}</label>
        <input 
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm font-mono" 
          onChange={e => { setBaseAmount(parseFloat(e.target.value)); }} 
          type="number"
          value={baseAmount}
        />
      </div>
      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.incentives.effective_from", { defaultValue: "EFFECTIVE FROM" })}</label>
        <input 
          className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm" 
          onChange={e => { setEffectiveFrom(e.target.value); }} 
          type="date"
          value={effectiveFrom}
        />
      </div>
    </div>
  );
}

function BonusPolicyForm({
  onCancel,
  onSave,
  isPending
}: {
  isPending: boolean;
  onCancel: () => void;
  onSave: (data: Parameters<typeof createIncentivePolicy>[1]) => void;
}) {
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
    <div className="bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] p-10 border border-slate-200 dark:border-slate-700 shadow-inner animate-in slide-in-from-top-4 duration-500 space-y-10">
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
        <Button className="bg-transparent hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-500 dark:text-slate-400 px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] transition-all" 
          onClick={onCancel}
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
        <Button className="bg-slate-900 hover:bg-primary-700 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] shadow-2xl active:scale-95 transition-all flex items-center gap-3 disabled:opacity-50" 
          disabled={isPending}
          onClick={handleSave}
          variant="ghost"
        >
          {isPending ? <Plus className="animate-spin" size={18} /> : <Save size={18} />}
          {t("common.save_settings")}
        </Button>
      </div>
    </div>
  );
}

function BonusPolicyCard({
  policy,
  onClone,
  onDelete
}: {
  onClone: (id: string) => void;
  onDelete: (id: string) => void;
  policy: PayrollIncentivePolicyResponse;
}) {
  const { t } = useTranslation();
  const renderRules = (tier: IncentiveTier) => {
    if (!tier.rule_config) return <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase italic">Default</span>;
    
    // Option B: Array
    if (Array.isArray(tier.rule_config)) {
      if (tier.rule_config.length === 0) return <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase italic">Default</span>;
      return tier.rule_config.map((r: RuleCondition, i) => (
        <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase" key={i}>{r.metric.replace(/_/g, ' ')} {r.operator} {r.value}</span>
      ));
    }

    // Option A: Legacy flat object
    const config = tier.rule_config as Record<string, string | number | undefined | null>;
    const rules = [];
    if (config.max_late_early_count != null) rules.push(`Late < ${String(config.max_late_early_count)}`);
    if (config.max_unpaid_leave_days != null) rules.push(`Unpaid < ${String(config.max_unpaid_leave_days)}d`);
    if (config.max_sick_leave_days != null) rules.push(`Sick < ${String(config.max_sick_leave_days)}d`);
    if (config.max_total_leave_days != null) rules.push(`Total < ${String(config.max_total_leave_days)}d`);
    
    if (rules.length === 0) return <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase italic">Default</span>;
    return rules.map((r, i) => <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase" key={i}>{r}</span>);
  };

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-8 shadow-sm hover:shadow-md transition-all group relative overflow-hidden">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 relative z-10">
        <div className="flex items-center gap-6">
          <div className="w-16 h-16 rounded-[1.5rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50">
            <Target size={32} />
          </div>
          <div>
            <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">{policy.name}</Typography>
            <div className="flex items-center gap-4 mt-1.5">
              <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest bg-slate-50 dark:bg-slate-800 px-3 py-1 rounded-lg border border-slate-100 dark:border-slate-800">BASE: {new Intl.NumberFormat('vi-VN').format(policy.base_amount)} VND</span>
              {policy.is_active ? (
                <span className="flex items-center gap-1.5 text-[10px] font-bold text-emerald-600 dark:text-emerald-300 uppercase tracking-widest">
                  <CheckCircle2 size={12} /> {t("common.active")}
                </span>
              ) : (
                <span className="flex items-center gap-1.5 text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                  {t("common.inactive", { defaultValue: "INACTIVE" })}
                </span>
              )}
              {!!policy.effective_from && (
                <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800">
                  {policy.effective_from}
                </span>
              )}
            </div>
          </div>
        </div>

        <div className="flex flex-wrap gap-3">
          {[...policy.tiers].sort((a: IncentiveTier, b: IncentiveTier) => a.priority - b.priority).map((tier: IncentiveTier) => (
            <div className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 flex flex-col items-center min-w-[140px] shadow-inner" key={tier.id}>
              <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2">Tier {tier.priority}</span>
              <span className="text-sm font-bold text-primary-600 dark:text-primary-400 font-mono tracking-tighter">
                +{new Intl.NumberFormat('vi-VN').format(tier.payout_amount)}
              </span>
              <div className="mt-2 flex flex-col items-center gap-1 opacity-60">
                {renderRules(tier)}
              </div>
            </div>
          ))}
        </div>

        <div className="flex flex-col gap-2">
          <Button className="p-3 rounded-2xl text-slate-400 dark:text-slate-500 hover:text-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/50 transition-all active:scale-90 flex items-center justify-center" 
            onClick={() => { onClone(policy.id); }}
            title="Clone Policy"
            variant="ghost"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><rect height="13" rx="2" ry="2" width="13" x="9" y="9"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          </Button>
          <Button className="p-3 rounded-2xl text-slate-400 dark:text-slate-500 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950 transition-all active:scale-90 flex items-center justify-center" 
            onClick={() => { onDelete(policy.id); }}
            title="Delete Policy"
            variant="ghost"
          >
            <Trash2 size={20} />
          </Button>
        </div>
      </div>
      <Sparkles className="absolute -right-8 -bottom-8 w-40 h-40 text-primary-500 opacity-[0.03] -rotate-12 group-hover:rotate-0 transition-transform duration-1000 pointer-events-none" />
    </div>
  );
}

export default function BonusPolicyManager({ wsId }: BonusPolicyManagerProps) {
 
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isCreating, setIsCreating] = useState(false);



  const { data: policies = [], isLoading } = useQuery({
    queryKey: ["incentive-policies", wsId],
    queryFn: () => fetchIncentivePolicies(wsId),
  });

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createIncentivePolicy>[1]) => createIncentivePolicy(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      setIsCreating(false);
      toast.success(t("common.success"));
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteIncentivePolicy(wsId, id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      toast.success(t("common.deleted"));
    },
  });

  const cloneMutation = useMutation({
    mutationFn: (id: string) => cloneIncentivePolicy(wsId, id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      toast.success(t("common.success", { defaultValue: "Cloned successfully" }));
    },
  });



  return (
    <div className="space-y-10">
      <div className="flex items-center justify-between">
        <div>
          <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight flex items-center gap-3" variant="h2">
            <Sparkles className="text-primary-600 dark:text-primary-400" size={24} />
            {t("payroll.incentives.title", { defaultValue: "INCENTIVE & BONUS POLICIES" })}
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400 font-medium mt-1 uppercase tracking-widest text-[10px] opacity-70" variant="p">Manage Diligence, Performance and Safety bonuses</Typography>
        </div>
        <Button className={cn(
            "flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg active:scale-95",
            isCreating ? "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400" : "bg-primary-600 text-white hover:bg-primary-700"
          )} 
          onClick={() => { setIsCreating(!isCreating); }}
          variant="ghost"
        >
          {isCreating ? t("common.cancel") : <><Plus size={16} /> {t("payroll.incentives.add_btn", { defaultValue: "NEW POLICY" })}</>}
        </Button>
      </div>

      {isCreating ? (
        <BonusPolicyForm 
          isPending={createMutation.isPending}
          onCancel={() => { setIsCreating(false); }}
          onSave={(data) => { createMutation.mutate(data); }}
        />
      ) : null}

      <div className="grid grid-cols-1 gap-6">
        {policies.map(policy => (
          <BonusPolicyCard 
            key={policy.id}
            onClone={(id) => { cloneMutation.mutate(id); }}
            onDelete={(id) => { deleteMutation.mutate(id); }}
            policy={policy}
          />
        ))}

        {!isLoading && policies.length === 0 && !isCreating ? (
          <div className="py-24 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><Sparkles className="text-slate-200 dark:text-slate-700" size={32} /></div>
            <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200" variant="h3">{t("payroll.incentives.no_policies", { defaultValue: "NO INCENTIVE POLICIES" })}</Typography>
            <Typography className="text-slate-500 dark:text-slate-400 text-sm mt-1 uppercase font-bold tracking-widest opacity-50" variant="p">Configure your first bonus rule to reward diligent staff</Typography>
          </div>
        ) : null}
      </div>
    </div>
  );
}
