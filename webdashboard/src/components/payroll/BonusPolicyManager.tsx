import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Sparkles, 
  Plus, 
  Trash2, 
  Info, 
  Save,
  CheckCircle2,
  AlertCircle,
  TrendingUp,
  Target,
  Clock
} from "lucide-react";
import { cn } from "@/lib/utils";
import { fetchIncentivePolicies, createIncentivePolicy, deleteIncentivePolicy } from "@/lib/api/payroll";
import { useToast } from "@/components/Toast";

interface BonusPolicyManagerProps {
  wsId: string;
}

export default function BonusPolicyManager({ wsId }: BonusPolicyManagerProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isCreating, setIsCreating] = useState(false);

  // Form State
  const [name, setName] = useState("");
  const [baseAmount, setBaseAmount] = useState(300000);
  const [payoutType, setPayoutType] = useState<"fixed" | "percentage">("fixed");
  const [tiers, setTiers] = useState<any[]>([
    { priority: 1, max_total_leave_days: 1, max_late_early_count: 2, max_unpaid_leave_days: 0.5, max_sick_leave_days: 1, payout_amount: 150000 },
    { priority: 2, max_total_leave_days: null, max_late_early_count: null, max_unpaid_leave_days: null, max_sick_leave_days: null, payout_amount: 0 }
  ]);

  const { data: policies = [], isLoading } = useQuery({
    queryKey: ["incentive-policies", wsId],
    queryFn: () => fetchIncentivePolicies(wsId),
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createIncentivePolicy(wsId, data),
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

  const handleRemoveTier = (idx: number) => {
    setTiers(tiers.filter((_, i) => i !== idx));
  };

  const updateTier = (idx: number, field: string, val: any) => {
    const next = [...tiers];
    next[idx][field] = val === "" ? null : val;
    setTiers(next);
  };

  const handleSave = () => {
    if (!name) return toast.error(t("payroll.incentives.name_required"));
    createMutation.mutate({
      name,
      base_amount: baseAmount,
      payout_type: payoutType,
      tiers: tiers.map(t => ({
        ...t,
        max_total_leave_days: t.max_total_leave_days !== null ? parseFloat(t.max_total_leave_days) : null,
        max_late_early_count: t.max_late_early_count !== null ? parseInt(t.max_late_early_count) : null,
        max_unpaid_leave_days: t.max_unpaid_leave_days !== null ? parseFloat(t.max_unpaid_leave_days) : null,
        max_sick_leave_days: t.max_sick_leave_days !== null ? parseFloat(t.max_sick_leave_days) : null,
        payout_amount: parseFloat(t.payout_amount)
      }))
    });
  };

  return (
    <div className="space-y-10">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-slate-700 uppercase tracking-tight flex items-center gap-3">
            <Sparkles className="text-primary-600" size={24} />
            {t("payroll.incentives.title", { defaultValue: "INCENTIVE & BONUS POLICIES" })}
          </h2>
          <p className="text-sm text-slate-500 font-medium mt-1 uppercase tracking-widest text-[10px] opacity-70">Manage Diligence, Performance and Safety bonuses</p>
        </div>
        <button 
          onClick={() => setIsCreating(!isCreating)}
          className={cn(
            "flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg active:scale-95",
            isCreating ? "bg-slate-100 text-slate-500" : "bg-primary-600 text-white hover:bg-primary-700"
          )}
        >
          {isCreating ? t("common.cancel") : <><Plus size={16} /> {t("payroll.incentives.add_btn", { defaultValue: "NEW POLICY" })}</>}
        </button>
      </div>

      {isCreating && (
        <div className="bg-slate-50 rounded-[2.5rem] p-10 border border-slate-200 shadow-inner animate-in slide-in-from-top-4 duration-500 space-y-10">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("payroll.incentives.policy_name")}</label>
              <input 
                type="text" 
                value={name} 
                onChange={e => setName(e.target.value)}
                placeholder="e.g. Diligence Bonus"
                className="w-full bg-white border border-slate-200 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm"
              />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("payroll.incentives.base_amount")}</label>
              <input 
                type="number" 
                value={baseAmount} 
                onChange={e => setBaseAmount(parseFloat(e.target.value))}
                className="w-full bg-white border border-slate-200 rounded-2xl px-6 py-4 outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm font-mono"
              />
            </div>
          </div>

          <div className="space-y-6">
            <div className="flex items-center justify-between px-1">
              <h3 className="text-xs font-bold text-slate-500 uppercase tracking-[0.2em]">{t("payroll.incentives.tiers_config", { defaultValue: "EVALUATION TIERS" })}</h3>
              <button onClick={handleAddTier} className="text-primary-600 hover:text-primary-700 font-bold text-[10px] uppercase tracking-widest flex items-center gap-1.5"><Plus size={14} /> {t("common.add")}</button>
            </div>

            <div className="space-y-4">
              {tiers.map((tier, idx) => (
                <div key={idx} className="bg-white p-8 rounded-3xl border border-slate-100 shadow-sm flex flex-col items-stretch gap-8 group relative">
                  <div className="flex items-center justify-between">
                    <div className="w-10 h-10 rounded-xl bg-slate-100 flex items-center justify-center font-bold text-slate-500 text-xs shadow-inner">#{idx + 1}</div>
                    <button 
                      onClick={() => handleRemoveTier(idx)}
                      className="w-10 h-10 rounded-xl text-slate-300 hover:text-rose-500 hover:bg-rose-50 transition-all flex items-center justify-center"
                    >
                      <Trash2 size={18} />
                    </button>
                  </div>
                  
                  <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-6">
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-slate-500 uppercase tracking-widest flex items-center gap-1.5"><Clock size={10} /> Max Late/Early</label>
                      <input 
                        type="number" 
                        value={tier.max_late_early_count || ""} 
                        onChange={e => updateTier(idx, "max_late_early_count", e.target.value)}
                        placeholder="Any"
                        className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-2.5 outline-none focus:bg-white transition-all font-bold text-xs"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-slate-500 uppercase tracking-widest flex items-center gap-1.5"><AlertCircle size={10} /> Max Unpaid</label>
                      <input 
                        type="number" 
                        value={tier.max_unpaid_leave_days || ""} 
                        onChange={e => updateTier(idx, "max_unpaid_leave_days", e.target.value)}
                        placeholder="Any"
                        className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-2.5 outline-none focus:bg-white transition-all font-bold text-xs"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-slate-500 uppercase tracking-widest flex items-center gap-1.5"><AlertCircle size={10} /> Max Sick</label>
                      <input 
                        type="number" 
                        value={tier.max_sick_leave_days || ""} 
                        onChange={e => updateTier(idx, "max_sick_leave_days", e.target.value)}
                        placeholder="Any"
                        className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-2.5 outline-none focus:bg-white transition-all font-bold text-xs"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-slate-500 uppercase tracking-widest flex items-center gap-1.5"><AlertCircle size={10} /> Max Total Days</label>
                      <input 
                        type="number" 
                        value={tier.max_total_leave_days || ""} 
                        onChange={e => updateTier(idx, "max_total_leave_days", e.target.value)}
                        placeholder="Any"
                        className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-2.5 outline-none focus:bg-white transition-all font-bold text-xs"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-[9px] font-bold text-primary-600 uppercase tracking-widest flex items-center gap-1.5"><TrendingUp size={10} /> Payout</label>
                      <input 
                        type="number" 
                        value={tier.payout_amount} 
                        onChange={e => updateTier(idx, "payout_amount", e.target.value)}
                        className="w-full bg-primary-50/50 border border-primary-100 rounded-xl px-4 py-2.5 outline-none focus:bg-white transition-all font-bold text-xs font-mono text-primary-700"
                      />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="flex justify-end pt-4">
            <button 
              onClick={handleSave}
              disabled={createMutation.isPending}
              className="bg-slate-900 hover:bg-primary-700 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] shadow-2xl active:scale-95 transition-all flex items-center gap-3 disabled:opacity-50"
            >
              {createMutation.isPending ? <Plus className="animate-spin" size={18} /> : <Save size={18} />}
              {t("common.save_settings")}
            </button>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 gap-6">
        {policies.map(policy => (
          <div key={policy.id} className="bg-white border border-slate-100 rounded-[2.5rem] p-8 shadow-sm hover:shadow-md transition-all group relative overflow-hidden">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 relative z-10">
              <div className="flex items-center gap-6">
                <div className="w-16 h-16 rounded-[1.5rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-inner border border-primary-100/50">
                  <Target size={32} />
                </div>
                <div>
                  <h3 className="text-xl font-bold text-slate-700 uppercase tracking-tight">{policy.name}</h3>
                  <div className="flex items-center gap-4 mt-1.5">
                    <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest bg-slate-50 px-3 py-1 rounded-lg border border-slate-100">BASE: {new Intl.NumberFormat('vi-VN').format(policy.base_amount)} VND</span>
                    <span className="flex items-center gap-1.5 text-[10px] font-bold text-emerald-600 uppercase tracking-widest">
                      <CheckCircle2 size={12} /> {t("common.active")}
                    </span>
                  </div>
                </div>
              </div>

              <div className="flex flex-wrap gap-3">
                {policy.tiers?.sort((a, b) => a.priority - b.priority).map((tier, tidx) => (
                  <div key={tier.id} className="bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 flex flex-col items-center min-w-[140px] shadow-inner">
                    <span className="text-[8px] font-bold text-slate-500 uppercase tracking-widest mb-2">Tier {tier.priority}</span>
                    <span className="text-sm font-bold text-primary-600 font-mono tracking-tighter">
                      +{new Intl.NumberFormat('vi-VN').format(tier.payout_amount)}
                    </span>
                    <div className="mt-2 flex flex-col items-center gap-1 opacity-60">
                      {tier.max_late_early_count !== null && <span className="text-[7px] font-bold text-slate-500 uppercase">Late &lt; {tier.max_late_early_count}</span>}
                      {tier.max_unpaid_leave_days !== null && <span className="text-[7px] font-bold text-slate-500 uppercase">Unpaid &lt; {tier.max_unpaid_leave_days}d</span>}
                      {tier.max_sick_leave_days !== null && <span className="text-[7px] font-bold text-slate-500 uppercase">Sick &lt; {tier.max_sick_leave_days}d</span>}
                      {tier.max_total_leave_days !== null && <span className="text-[7px] font-bold text-slate-500 uppercase">Total &lt; {tier.max_total_leave_days}d</span>}
                      {tier.max_late_early_count === null && tier.max_unpaid_leave_days === null && tier.max_sick_leave_days === null && tier.max_total_leave_days === null && (
                        <span className="text-[7px] font-bold text-slate-500 uppercase italic">Default</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              <button 
                onClick={() => deleteMutation.mutate(policy.id)}
                className="p-4 rounded-2xl text-slate-300 hover:text-rose-500 hover:bg-rose-50 transition-all active:scale-90"
              >
                <Trash2 size={20} />
              </button>
            </div>
            <Sparkles className="absolute -right-8 -bottom-8 w-40 h-40 text-primary-500 opacity-[0.03] -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
          </div>
        ))}

        {!isLoading && policies.length === 0 && !isCreating && (
          <div className="py-24 text-center border-2 border-dashed border-slate-100 rounded-[3rem] bg-slate-50/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100"><Sparkles size={32} className="text-slate-200" /></div>
            <h3 className="text-lg font-bold text-slate-700">{t("payroll.incentives.no_policies", { defaultValue: "NO INCENTIVE POLICIES" })}</h3>
            <p className="text-slate-500 text-sm mt-1 uppercase font-bold tracking-widest opacity-50">Configure your first bonus rule to reward diligent staff</p>
          </div>
        )}
      </div>
    </div>
  );
}
