import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, LayoutList } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { fetchPayrollRuleTiers, createPayrollRuleTier } from "@/lib/api/payroll";
import { PayrollRuleTier } from "@/lib/api/types";


interface Props {
  onClose?: () => void;
  ruleId: string;
  wsId: string;
}

export default function RuleTierManager({ wsId, ruleId, onClose: _onClose }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: tiers, isLoading } = useQuery({
    queryKey: ["payroll-rule-tiers", ruleId],
    queryFn: () => fetchPayrollRuleTiers(wsId, ruleId),
    enabled: !!ruleId,
  });

  const [newTier, setNewTier] = useState<Partial<PayrollRuleTier>>({
    min_amount: 0,
    max_amount: undefined,
    rate: 0,
    fixed_amount: 0,
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<PayrollRuleTier>) => createPayrollRuleTier(wsId, ruleId, { ...data, tier_order: (tiers?.length ?? 0) + 1 }),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["payroll-rule-tiers", ruleId] });
      setNewTier({ min_amount: 0, max_amount: undefined, rate: 0, fixed_amount: 0 });
    },
  });

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    createMutation.mutate(newTier);
  };

  return (
    <div className="space-y-6">
      <div className="bg-slate-50 dark:bg-slate-800/50 rounded-2xl p-6 border border-slate-100 dark:border-slate-800">
        <div className="flex items-center gap-3 mb-4">
          <div className="w-8 h-8 rounded-lg bg-primary-100 dark:bg-primary-900/30 text-primary-600 flex items-center justify-center">
            <LayoutList size={16} />
          </div>
          <div>
            <h3 className="font-bold text-sm uppercase tracking-widest text-slate-700 dark:text-slate-200">{t("payroll.rule_tiers.title")}</h3>
            <p className="text-[10px] text-slate-500 uppercase tracking-widest mt-1">{t("payroll.rule_tiers.subtitle")}</p>
          </div>
        </div>

        {isLoading ? (
          <div className="p-10 text-center text-slate-500">{t("payroll.rule_tiers.loading")}</div>
        ) : (
          <div className="space-y-2">
            {tiers?.map((tier) => (
              <div className="flex items-center justify-between p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm" key={tier.id}>
                <div className="flex items-center gap-4">
                  <span className="w-6 h-6 rounded-full bg-slate-100 dark:bg-slate-800 flex items-center justify-center text-xs font-bold text-slate-500">{tier.tier_order}</span>
                  <div className="text-sm font-bold text-slate-700 dark:text-slate-200 font-mono">
                    {tier.min_amount.toLocaleString()} ₫ - {tier.max_amount ? `${tier.max_amount.toLocaleString()} ₫` : '∞'}
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  {tier.rate !== undefined && tier.rate > 0 && <span className="text-xs font-bold text-primary-600 bg-primary-50 px-2 py-1 rounded-md">{tier.rate}%</span>}
                  {tier.fixed_amount !== undefined && tier.fixed_amount > 0 && <span className="text-xs font-bold text-rose-600 bg-rose-50 px-2 py-1 rounded-md">-{tier.fixed_amount.toLocaleString()} ₫</span>}
                  <button className="text-slate-400 hover:text-rose-500 transition-colors"><Trash2 size={16} /></button>
                </div>
              </div>
            ))}

            {/* Add new tier form */}
            <form className="mt-4 p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-700 shadow-inner flex items-end gap-4" onSubmit={handleSubmit}>
              <div className="flex-1 space-y-2">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("payroll.rule_tiers.fields.min_amount")}</label>
                <input className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm font-bold" onChange={e => { setNewTier({...newTier, min_amount: parseFloat(e.target.value)}); }} required type="number" value={newTier.min_amount} />
              </div>
              <div className="flex-1 space-y-2">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("payroll.rule_tiers.fields.max_amount")}</label>
                <input className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm font-bold" onChange={e => { setNewTier({...newTier, max_amount: e.target.value ? parseFloat(e.target.value) : undefined}); }} type="number" value={newTier.max_amount ?? ""} />
              </div>
              <div className="flex-1 space-y-2">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("payroll.rule_tiers.fields.rate")}</label>
                <input className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm font-bold" onChange={e => { setNewTier({...newTier, rate: e.target.value ? parseFloat(e.target.value) : 0}); }} step="0.01" type="number" value={newTier.rate ?? ""} />
              </div>
              <div className="flex-1 space-y-2">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("payroll.rule_tiers.fields.fixed_deduction")}</label>
                <input className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm font-bold" onChange={e => { setNewTier({...newTier, fixed_amount: e.target.value ? parseFloat(e.target.value) : 0}); }} type="number" value={newTier.fixed_amount ?? ""} />
              </div>
              <button className="bg-primary-600 text-white p-2.5 rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50" disabled={createMutation.isPending} type="submit">
                <Plus size={16} />
              </button>
            </form>
          </div>
        )}
      </div>
    </div>
  );
}
