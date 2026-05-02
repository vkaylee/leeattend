import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Sparkles, 
  Plus, 
  Trash2, 
  Save,
  CheckCircle2,
  AlertCircle,
  TrendingUp,
  Target,
  Clock
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/Toast";
import { fetchIncentivePolicies, createIncentivePolicy, deleteIncentivePolicy } from "@/lib/api/payroll";
import { cn } from "@/lib/utils";

interface BonusPolicyManagerProps {
  wsId: string;
}

interface TierInput {
  [key: string]: string | number | null;
  max_late_early_count: string | number | null;
  max_sick_leave_days: string | number | null;
  max_total_leave_days: string | number | null;
  max_unpaid_leave_days: string | number | null;
  payout_amount: string | number;
  priority: number;
}

export default function BonusPolicyManager({ wsId }: BonusPolicyManagerProps) {
 
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isCreating, setIsCreating] = useState(false);

  // Form State
  const [name, setName] = useState("");
  const [baseAmount, setBaseAmount] = useState(300000);
  const [payoutType, _setPayoutType] = useState<"fixed" | "percentage">("fixed");
  const [tiers, setTiers] = useState<TierInput[]>([
    { priority: 1, max_total_leave_days: 1, max_late_early_count: 2, max_unpaid_leave_days: 0.5, max_sick_leave_days: 1, payout_amount: 150000 },
 
    { priority: 2, max_total_leave_days: null, max_late_early_count: null, max_unpaid_leave_days: null, max_sick_leave_days: null, payout_amount: 0 }
  ]);

  const { data: policies = [], isLoading } = useQuery({
    queryKey: ["incentive-policies", wsId],
    queryFn: () => fetchIncentivePolicies(wsId),
  });

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createIncentivePolicy>[1]) => createIncentivePolicy(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      setIsCreating(false);
      toast.success(t("common.success"));
      // Reset form
      setName(""); setBaseAmount(300000);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteIncentivePolicy(wsId, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      toast.success(t("common.deleted"));
    },
  });

  const handleAddTier = () => {
    setTiers([...tiers, { 
      priority: tiers.length + 1, 
      max_total_leave_days: null, 
      max_late_early_count: null,
      max_unpaid_leave_days: null,
 
      max_sick_leave_days: null,
      payout_amount: 0 
    }]);
  };

  const handleRemoveTier = (index: number) => {
    setTiers(tiers.filter((_, index_) => index_ !== index));
  };

  const updateTier = (index: number, field: string, value: string | number | null) => {
    const next = [...tiers];
    next[index][field] = value === "" ? null : value;
    setTiers(next);
  };

  const handleSave = () => {
    if (!name) { toast.error(t("payroll.incentives.name_required")); return; }
    createMutation.mutate({
      name,
      base_amount: baseAmount,
      payout_type: payoutType,
      tiers: tiers.map(t => ({
        ...t,
        max_total_leave_days: t.max_total_leave_days !== null && t.max_total_leave_days !== "" ? Number(t.max_total_leave_days) : null,
        max_late_early_count: t.max_late_early_count !== null && t.max_late_early_count !== "" ? Number(t.max_late_early_count) : null,
        max_unpaid_leave_days: t.max_unpaid_leave_days !== null && t.max_unpaid_leave_days !== "" ? Number(t.max_unpaid_leave_days) : null,
        max_sick_leave_days: t.max_sick_leave_days !== null && t.max_sick_leave_days !== "" ? Number(t.max_sick_leave_days) : null,
        payout_amount: Number(t.payout_amount)
      })) as Parameters<typeof createIncentivePolicy>[1]["tiers"]
    });
  };

  return (
    <div className="space-y-10">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight flex items-center gap-3">
            <Sparkles className="text-primary-600 dark:text-primary-400" size={24} />
            {t("payroll.incentives.title", { defaultValue: "INCENTIVE & BONUS POLICIES" })}
          </h2>
          <p className="text-sm text-slate-500 dark:text-slate-400 font-medium mt-1 uppercase tracking-widest text-[10px] opacity-70">Manage Diligence, Performance and Safety bonuses</p>
        </div>
        <button 
          className={cn(
            "flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg active:scale-95",
            isCreating ? "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400" : "bg-primary-600 text-white hover:bg-primary-700"
          )}
          onClick={() => { setIsCreating(!isCreating); }}
        >
          {isCreating ? t("common.cancel") : <><Plus size={16} /> {t("payroll.incentives.add_btn", { defaultValue: "NEW POLICY" })}</>}
        </button>
      </div>

      {!!isCreating && (
        <div className="bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] p-10 border border-slate-200 dark:border-slate-700 shadow-inner animate-in slide-in-from-top-4 duration-500 space-y-10">
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
          </div>

          <div className="space-y-6">
            <div className="flex items-center justify-between px-1">
              <h3 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("payroll.incentives.tiers_config", { defaultValue: "EVALUATION TIERS" })}</h3>
              <button className="text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:text-primary-300 font-bold text-[10px] uppercase tracking-widest flex items-center gap-1.5" onClick={handleAddTier}><Plus size={14} /> {t("common.add")}</button>
            </div>

            <div className="space-y-4">
              {tiers.map((tier, index) => (
                <div className="bg-white dark:bg-slate-900 p-8 rounded-3xl border border-slate-100 dark:border-slate-800 shadow-sm flex flex-col items-stretch gap-8 group relative" key={index}>
                  <div className="flex items-center justify-between">
                    <div className="w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-700 flex items-center justify-center font-bold text-slate-500 dark:text-slate-400 text-xs shadow-inner">#{index + 1}</div>
                    <button 
 
                      className="w-10 h-10 rounded-xl text-slate-300 dark:text-slate-600 hover:text-rose-500 hover:bg-rose-50 dark:bg-rose-950 transition-all flex items-center justify-center"
                      onClick={() => { handleRemoveTier(index); }}
                    >
                      <Trash2 size={18} />
                    </button>
                  </div>
                  
                  <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-6">
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-1.5" htmlFor={`late-${index}`}><Clock size={10} /> Max Late/Early</label>
                      { }
                      <input 
                        className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-2.5 outline-none focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-xs"
                        id={`late-${index}`} 
                        onChange={e => { updateTier(index, "max_late_early_count", e.target.value); }} 
                        placeholder="Any"
                        type="number"
                        value={tier.max_late_early_count || ""}
                      />
                    </div>
                    <div className="space-y-2">
                      { }
                      <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-1.5" htmlFor={`unpaid-${index}`}><AlertCircle size={10} /> Max Unpaid</label>
                      <input 
                        className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-2.5 outline-none focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-xs"
                        id={`unpaid-${index}`} 
                        onChange={e => { updateTier(index, "max_unpaid_leave_days", e.target.value); }} 
                        placeholder="Any"
                        type="number"
                        value={tier.max_unpaid_leave_days || ""}
                      />
                    </div>
                    { }
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-1.5" htmlFor={`sick-${index}`}><AlertCircle size={10} /> Max Sick</label>
                      <input 
                        className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-2.5 outline-none focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-xs"
                        id={`sick-${index}`} 
                        onChange={e => { updateTier(index, "max_sick_leave_days", e.target.value); }} 
                        placeholder="Any"
                        type="number"
                        value={tier.max_sick_leave_days || ""}
                      />
                      { }
                    </div>
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-1.5" htmlFor={`total-${index}`}><AlertCircle size={10} /> Max Total Days</label>
                      <input 
                        className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-2.5 outline-none focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-xs"
                        id={`total-${index}`} 
                        onChange={e => { updateTier(index, "max_total_leave_days", e.target.value); }} 
                        placeholder="Any"
                        type="number"
                        value={tier.max_total_leave_days || ""}
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest flex items-center gap-1.5" htmlFor={`payout-${index}`}><TrendingUp size={10} /> Payout</label>
                      <input 
                        className="w-full bg-primary-50 dark:bg-primary-950/50 border border-primary-100 dark:border-primary-800 rounded-xl px-4 py-2.5 outline-none focus:bg-white dark:focus:bg-slate-900 transition-all font-bold text-xs font-mono text-primary-700 dark:text-primary-300"
                        id={`payout-${index}`} 
                        onChange={e => { updateTier(index, "payout_amount", e.target.value); }} 
                        type="number"
                        value={tier.payout_amount}
                      />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="flex justify-end pt-4">
            <button 
              className="bg-slate-900 hover:bg-primary-700 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] shadow-2xl active:scale-95 transition-all flex items-center gap-3 disabled:opacity-50"
              disabled={createMutation.isPending}
              onClick={handleSave}
            >
              {createMutation.isPending ? <Plus className="animate-spin" size={18} /> : <Save size={18} />}
              {t("common.save_settings")}
            </button>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 gap-6">
        {policies.map(policy => (
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-8 shadow-sm hover:shadow-md transition-all group relative overflow-hidden" key={policy.id}>
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 relative z-10">
              <div className="flex items-center gap-6">
                <div className="w-16 h-16 rounded-[1.5rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50">
                  <Target size={32} />
                </div>
                <div>
                  <h3 className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">{policy.name}</h3>
                  <div className="flex items-center gap-4 mt-1.5">
                    <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest bg-slate-50 dark:bg-slate-800 px-3 py-1 rounded-lg border border-slate-100 dark:border-slate-800">BASE: {new Intl.NumberFormat('vi-VN').format(policy.base_amount)} VND</span>
                    <span className="flex items-center gap-1.5 text-[10px] font-bold text-emerald-600 dark:text-emerald-300 uppercase tracking-widest">
                      <CheckCircle2 size={12} /> {t("common.active")}
                    </span>
                  </div>
                </div>
              </div>

              <div className="flex flex-wrap gap-3">
                {policy.tiers?.sort((a, b) => a.priority - b.priority).map((tier, _tidx) => (
                  <div className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 flex flex-col items-center min-w-[140px] shadow-inner" key={tier.id}>
                    <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2">Tier {tier.priority}</span>
                    <span className="text-sm font-bold text-primary-600 dark:text-primary-400 font-mono tracking-tighter">
                      +{new Intl.NumberFormat('vi-VN').format(tier.payout_amount)}
                    </span>
                    <div className="mt-2 flex flex-col items-center gap-1 opacity-60">
                      {tier.max_late_early_count !== null && <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase">Late &lt; {tier.max_late_early_count}</span>}
                      {tier.max_unpaid_leave_days !== null && <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase">Unpaid &lt; {tier.max_unpaid_leave_days}d</span>}
                      {tier.max_sick_leave_days !== null && <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase">Sick &lt; {tier.max_sick_leave_days}d</span>}
                      {tier.max_total_leave_days !== null && <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase">Total &lt; {tier.max_total_leave_days}d</span>}
                      {tier.max_late_early_count === null && tier.max_unpaid_leave_days === null && tier.max_sick_leave_days === null && tier.max_total_leave_days === null && (
                        <span className="text-[7px] font-bold text-slate-500 dark:text-slate-400 uppercase italic">Default</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              <button 
                className="p-4 rounded-2xl text-slate-300 dark:text-slate-600 hover:text-rose-500 hover:bg-rose-50 dark:bg-rose-950 transition-all active:scale-90"
                onClick={() => { deleteMutation.mutate(policy.id); }}
              >
                <Trash2 size={20} />
              </button>
            </div>
            <Sparkles className="absolute -right-8 -bottom-8 w-40 h-40 text-primary-500 opacity-[0.03] -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
          </div>
        ))}

        {!isLoading && policies.length === 0 && !isCreating && (
          <div className="py-24 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><Sparkles className="text-slate-200 dark:text-slate-700" size={32} /></div>
            <h3 className="text-lg font-bold text-slate-700 dark:text-slate-200">{t("payroll.incentives.no_policies", { defaultValue: "NO INCENTIVE POLICIES" })}</h3>
            <p className="text-slate-500 dark:text-slate-400 text-sm mt-1 uppercase font-bold tracking-widest opacity-50">Configure your first bonus rule to reward diligent staff</p>
          </div>
        )}
      </div>
    </div>
  );
}
