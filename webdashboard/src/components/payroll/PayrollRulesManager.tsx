import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, CheckCircle2, CircleDashed, FileText, Trash2, Percent, Calculator } from "lucide-react";
import { fetchPayrollRules, createPayrollRule, fetchSalaryBaseGroups } from "@/lib/api/payroll";
import { useToast } from "@/components/ui/Toast";
import { useTranslation } from "react-i18next";
import { PayrollRuleDefinition } from "@/lib/api/types";
import Modal from "@/components/ui/Modal";
import { cn } from "@/lib/utils";
import RuleTierManager from "./RuleTierManager";
import RuleAssignmentManager from "./RuleAssignmentManager";

interface Props {
  wsId: string;
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
  
  const [formData, setFormData] = useState<Partial<PayrollRuleDefinition>>({ name: "", description: "", code: "", category: "statutory_deduction", applies_to: "employee", calculation_method: "percentage", is_mandatory: true, display_order: 1, rule_config: { rate: 0, base: "gross" } });

  const createMutation = useMutation({
    mutationFn: (data: Partial<PayrollRuleDefinition>) => createPayrollRule(wsId, data),
    onSuccess: () => { toast.success(t("common.success")); queryClient.invalidateQueries({ queryKey: ["payroll-rules"] }); setIsModalOpen(false); setFormData({ name: "", description: "", code: "", category: "statutory_deduction", applies_to: "employee", calculation_method: "percentage", is_mandatory: true, display_order: 1, rule_config: { rate: 0, base: "gross" } }); },
  });

  const handleSubmit = (e: React.FormEvent) => { e.preventDefault(); createMutation.mutate(formData); };

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden animate-in fade-in duration-700">
      <div className="p-10 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between bg-slate-50 dark:bg-slate-800/30 px-12">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-rose-50 dark:bg-rose-950 flex items-center justify-center text-rose-600 dark:text-rose-300 shadow-inner border border-rose-100 dark:border-rose-800/50"><FileText size={24} /></div>
          <div>
            <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-none">Statutory Rules</h2>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">Manage dynamic statutory deductions and rules</p>
          </div>
        </div>
        <button onClick={() => setIsModalOpen(true)} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95"><Plus size={18} /> {t("common.add").toUpperCase()}</button>
      </div>

      <div className="divide-y divide-slate-50 dark:divide-slate-800 px-2">
        {isLoading ? (
          <div data-testid="loader" className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : rules?.length === 0 ? (
          <div className="p-32 text-center bg-slate-50 dark:bg-slate-800/30 m-8 rounded-[3rem] border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><FileText size={32} className="text-slate-200 dark:text-slate-700" /></div>
            <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">No Statutory Rules Configured</p>
          </div>
        ) : (
          rules?.map((rule) => (
            <div key={rule.id} className="border-b border-slate-50 dark:border-slate-800/50 last:border-0">
              <div 
                className={cn("p-8 px-10 flex items-center justify-between transition-all group relative m-2 rounded-3xl cursor-pointer", expandedRuleId === rule.id ? "bg-slate-50 dark:bg-slate-800/80 shadow-inner" : "hover:bg-slate-50 dark:hover:bg-slate-800/50")}
                onClick={() => setExpandedRuleId(expandedRuleId === rule.id ? null : rule.id)}
              >
                <div className="flex items-center gap-6">
                  <div className="w-14 h-14 rounded-2xl bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 font-bold text-lg shadow-sm group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 transition-all uppercase">{rule.code.slice(0, 2)}</div>
                  <div>
                    <p className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{rule.name}</p>
                    <p className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-2 opacity-70">
                      {rule.category} • {rule.calculation_method} 
                      {rule.calculation_method === "percentage" && ` (${rule.rule_config?.rate}%)`}
                      {rule.calculation_method === "fixed" && ` (Fixed Amount)`}
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center gap-10">
                  <div className="flex flex-col items-end gap-2">
                    <span className={cn("text-[9px] font-bold uppercase tracking-widest px-3 py-1 rounded-full border shadow-sm", rule.is_mandatory ? 'bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>
                      {rule.is_mandatory ? "MANDATORY" : "OPTIONAL"}
                    </span>
                    <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-50 italic">{rule.applies_to.toUpperCase()}</span>
                  </div>
                  <button className="p-2.5 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 active:scale-90" onClick={(e) => e.stopPropagation()}><Trash2 size={18} /></button>
                </div>
              </div>

              {/* Expanded details section */}
              {expandedRuleId === rule.id && (
                <div className="px-12 pb-8 pt-2 animate-in slide-in-from-top-4 duration-300 space-y-6">
                  {rule.calculation_method === "tiered" && (
                    <RuleTierManager wsId={wsId} ruleId={rule.id} />
                  )}
                  <RuleAssignmentManager wsId={wsId} ruleId={rule.id} calculationMethod={rule.calculation_method} />
                </div>
              )}
            </div>
          ))
        )}
      </div>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title="Add Rule" subtitle="Create a new payroll rule" icon={<FileText size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label htmlFor="rule-name" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.name")}</label>
              <input id="rule-name" required type="text" placeholder="Social Insurance" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" value={formData.name} onChange={e => setFormData({...formData, name: e.target.value})} />
            </div>
            <div className="space-y-3 px-1">
              <label htmlFor="rule-code" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Code</label>
              <input id="rule-code" required type="text" placeholder="BHXH" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" value={formData.code} onChange={e => setFormData({...formData, code: e.target.value.toUpperCase()})} />
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label htmlFor="rule-category" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Category</label>
              <select id="rule-category" required className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" value={formData.category} onChange={e => setFormData({...formData, category: e.target.value as any})}>
                <option value="statutory_deduction">Statutory Deduction</option>
                <option value="allowance">Allowance</option>
                <option value="bonus">Bonus</option>
                <option value="deduction">Deduction</option>
              </select>
            </div>
            <div className="space-y-3 px-1">
              <label htmlFor="rule-calc" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Calculation Method</label>
              <select id="rule-calc" required className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" value={formData.calculation_method} onChange={e => {
                const method = e.target.value as any;
                let newConfig = {};
                if (method === "percentage") newConfig = { rate: 0, base: "gross" };
                else if (method === "fixed") newConfig = { amount: 0, frequency: "monthly" };
                else if (method === "tiered") newConfig = { base: "taxable_income" };
                setFormData({...formData, calculation_method: method, rule_config: newConfig});
              }}>
                <option value="percentage">Percentage</option>
                <option value="fixed">Fixed</option>
                <option value="tiered">Tiered (e.g. Tax Brackets)</option>
              </select>
            </div>
          </div>

          <div className="bg-slate-50 dark:bg-slate-800/50 p-8 rounded-[2rem] border border-slate-100 dark:border-slate-800 space-y-6">
            <div className="flex items-center gap-3 mb-2">
              <Calculator size={18} className="text-primary-600 dark:text-primary-400" />
              <h3 className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-widest">Logic Configuration</h3>
            </div>
            
            {formData.calculation_method === "percentage" && (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="space-y-3">
                  <label htmlFor="rule-rate" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1"><Percent size={12}/> Rate (%)</label>
                  <input id="rule-rate" type="number" step="0.01" required className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm" value={formData.rule_config?.rate || ""} onChange={e => setFormData({...formData, rule_config: {...formData.rule_config, rate: parseFloat(e.target.value)}})} />
                </div>
                <div className="space-y-3">
                  <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Calculation Base</label>
                  <select required className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm uppercase tracking-tight" value={formData.rule_config?.base || "gross"} onChange={e => setFormData({...formData, rule_config: {...formData.rule_config, base: e.target.value}})}>
                    {bases?.map(b => (
                      <option key={b.slug} value={b.slug}>{b.name}</option>
                    ))}
                    {(!bases || bases.length === 0) && (
                      <>
                        <option value="gross">Gross Salary (Total)</option>
                        <option value="taxable_income">Taxable Income</option>
                      </>
                    )}
                  </select>
                </div>
              </div>
            )}

            {formData.calculation_method === "fixed" && (
              <div className="grid grid-cols-1 gap-8">
                <div className="space-y-3">
                  <label htmlFor="rule-fixed" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Fixed Amount (₫)</label>
                  <input id="rule-fixed" type="number" required className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm" value={formData.rule_config?.amount || ""} onChange={e => setFormData({...formData, rule_config: {...formData.rule_config, amount: parseFloat(e.target.value)}})} />
                </div>
              </div>
            )}

            {formData.calculation_method === "tiered" && (
              <div className="grid grid-cols-1 gap-8">
                <div className="space-y-3">
                  <label htmlFor="rule-tier-base" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Tier Base Amount</label>
                  <select id="rule-tier-base" required className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 outline-none transition-all shadow-sm uppercase tracking-tight" value={formData.rule_config?.base || "taxable_income"} onChange={e => setFormData({...formData, rule_config: {...formData.rule_config, base: e.target.value}})}>
                    {bases?.map(b => (
                      <option key={b.slug} value={b.slug}>{b.name}</option>
                    ))}
                    {(!bases || bases.length === 0) && (
                      <>
                        <option value="taxable_income">Taxable Income</option>
                        <option value="gross">Gross Salary (Total)</option>
                      </>
                    )}
                  </select>
                  <p className="text-xs text-slate-500 dark:text-slate-400 mt-2 ml-1 italic">Note: Tier brackets (e.g. 0-5M, 5-10M) will be configured after creating the rule.</p>
                </div>
              </div>
            )}
          </div>

          <div className="space-y-5">
            {[
              { key: "is_mandatory", label: "Mandatory Rule", desc: "Cannot be bypassed by default", icon: CheckCircle2 }
            ].map(field => (
              <label key={field.key} className={cn("flex items-center gap-6 p-8 rounded-[2rem] cursor-pointer transition-all group/check border-2 shadow-sm active:scale-[0.98]", 
                (formData as any)[field.key] ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800 shadow-xl shadow-primary-900/5" : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:bg-white dark:hover:bg-slate-900 hover:border-slate-100 dark:border-slate-800 shadow-inner"
              )}>
                <input type="checkbox" className="w-7 h-7 rounded-xl text-primary-600 dark:text-primary-400 focus:ring-primary-500 transition-all border-slate-200 dark:border-slate-700 shadow-inner cursor-pointer" checked={(formData as any)[field.key]} onChange={e => setFormData({...formData, [field.key]: e.target.checked})} />
                <div>
                  <p className="font-bold text-slate-800 dark:text-slate-100 text-sm uppercase tracking-tight group-hover/check:text-primary-600 dark:text-primary-400 transition-colors flex items-center gap-2"><field.icon size={14} className="opacity-50" /> {field.label}</p>
                  <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70 leading-relaxed max-w-sm">{field.desc}</p>
                </div>
              </label>
            ))}
          </div>

          <div className="flex gap-6 pt-4 border-t border-slate-50 dark:border-slate-800">
            <button type="button" onClick={() => setIsModalOpen(false)} className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" disabled={createMutation.isPending} className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50">{createMutation.isPending ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
