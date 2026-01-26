import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { fetchLeavePolicies, createLeavePolicy, deleteLeavePolicy, updateLeavePolicy, fetchLeaveTypes } from "@/lib/api/leaves";
import { LeavePolicy, LeaveType } from "@/lib/api/types";
import { Plus, Trash2, Clock, Edit2, ShieldCheck, Milestone, LayoutGrid } from "lucide-react";
import { useToast } from "@/components/Toast";
import Modal from "@/components/ui/Modal";
import { Combobox } from "@/components/ui/Combobox";
import { cn } from "@/lib/utils";
const FormRow = ({ label, children, info }: { label: string; children: React.ReactNode; info?: string }) => (
  <div className="grid grid-cols-1 md:grid-cols-12 gap-4 items-center group">
    <div className="md:col-span-4 md:text-right">
      <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] group-hover:text-primary-600 transition-colors">
        {label}
      </label>
    </div>
    <div className="md:col-span-8 flex flex-col">
      <div className="w-full max-w-md">
        {children}
      </div>
      {info && <p className="text-[9px] text-slate-400 italic font-medium ml-1 mt-1">{info}</p>}
    </div>
  </div>
);


interface LeavePolicyManagerProps {
  wsId: string;
}

export function LeavePolicyManager({ wsId }: LeavePolicyManagerProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [isCreating, setIsCreating] = useState(false);
  const [editingPolicy, setEditingPolicy] = useState<LeavePolicy | null>(null);

  const { data: policies = [], isLoading } = useQuery({
    queryKey: ["leave-policies", wsId],
    queryFn: () => fetchLeavePolicies(wsId),
  });

  const { data: leaveTypes = [] } = useQuery({
    queryKey: ["leave-types", wsId],
    queryFn: () => fetchLeaveTypes(wsId),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteLeavePolicy(wsId, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success("Policy deleted successfully");
    },
  });

  const mutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) => 
      editingPolicy ? updateLeavePolicy(wsId, editingPolicy.id, data) : createLeavePolicy(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success(editingPolicy ? "Policy updated" : "Policy created");
      setIsCreating(false);
      setEditingPolicy(null);
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const calculateAnnualEntitlement = (policy: LeavePolicy) => {
    const amount = policy.rule_config.accrual_amount;
    switch (policy.rule_config.frequency) {
      case "monthly": return amount * 12;
      case "quarterly": return amount * 4;
      case "yearly": return amount;
      default: return amount;
    }
  };

  if (isLoading) return <div className="p-12 text-center text-slate-400 animate-pulse font-bold uppercase tracking-widest text-[10px]">Loading policies...</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-end px-1">
         <button data-testid="add-policy-btn" onClick={() => setIsCreating(true)} className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100"><Plus size={14} /> Create Policy</button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {policies.length === 0 ? (
          <div className="col-span-full py-20 bg-slate-50 border-2 border-dashed border-slate-200 rounded-[2.5rem] flex flex-col items-center justify-center text-slate-400">
            <ShieldCheck size={40} className="mb-4 opacity-20" />
            <p className="text-xs font-bold uppercase tracking-widest">No policies defined yet</p>
          </div>
        ) : policies.map((policy) => {
          const type = leaveTypes.find(t => t.id === policy.leave_type_id);
          const bonusCount = policy.rule_config.seniority_bonus?.length || 0;
          return (
            <div key={policy.id} className="relative flex flex-col justify-between rounded-[2rem] border border-slate-200 bg-white p-7 shadow-sm hover:shadow-md transition-all group">
              <div>
                <div className="flex items-center justify-between">
                  <span className="inline-flex items-center rounded-lg bg-primary-50 px-2.5 py-1 text-[9px] font-bold text-primary-600 uppercase tracking-widest border border-primary-100 shadow-sm">
                    {type?.name || "Unknown"}
                  </span>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button data-testid="edit-policy-btn" onClick={() => setEditingPolicy(policy)} className="p-2 text-slate-400 hover:text-primary-600 bg-slate-50 rounded-xl border border-slate-100"><Edit2 size={16} /></button>
                    <button data-testid="delete-policy-btn" onClick={() => { if (confirm("Delete this policy?")) deleteMutation.mutate(policy.id); }} className="p-2 text-slate-400 hover:text-rose-600 bg-slate-50 rounded-xl border border-slate-100"><Trash2 size={16} /></button>
                  </div>
                </div>
                <h3 className="mt-5 text-lg font-bold text-slate-800 tracking-tight leading-none uppercase">{policy.name}</h3>
                
                <div className="mt-6 p-4 bg-slate-50 rounded-2xl border border-slate-100 space-y-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-slate-400"><Clock size={14} /><span className="text-[10px] font-bold uppercase tracking-widest">Base Accrual</span></div>
                    <span className="text-[10px] font-bold text-slate-600 uppercase tracking-widest">{policy.rule_config.accrual_amount} / {policy.rule_config.frequency}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-slate-400"><Milestone size={14} /><span className="text-[10px] font-bold uppercase tracking-widest">Bonus Steps</span></div>
                    <span className="text-[10px] font-bold text-primary-600 uppercase tracking-widest">{bonusCount} steps</span>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {(isCreating || editingPolicy) && (
        <PolicyModal
          isOpen={true}
          onClose={() => { setIsCreating(false); setEditingPolicy(null); }}
          leaveTypes={leaveTypes}
          policy={editingPolicy}
          onSave={(data: any) => mutation.mutate(data)}
          isSaving={mutation.isPending}
        />
      )}
    </div>
  );
}

function PolicyModal({ isOpen, onClose, leaveTypes, policy, onSave, isSaving }: any) {
  const [formData, setFormData] = useState({
    name: policy?.name || "",
    description: policy?.description || "",
    leave_type_id: policy?.leave_type_id || (leaveTypes.length > 0 ? leaveTypes[0].id : ""),
    accrual_amount: policy?.rule_config.accrual_amount || 1.0,
    frequency: policy?.rule_config.frequency || "monthly",
    carry_over_limit: policy?.rule_config.carry_over_limit || 5,
    min_work_days_threshold: policy?.rule_config.min_work_days_threshold || 14
  });

  const [seniorityBonus, setSeniorityBonus] = useState<{ months: number; bonus_amount: number }[]>(
    policy?.rule_config.seniority_bonus || []
  );

  const addBonusMilestone = () => {
    setSeniorityBonus([...seniorityBonus, { months: 12, bonus_amount: 1.0 }]);
  };

  const removeBonusMilestone = (index: number) => {
    setSeniorityBonus(seniorityBonus.filter((_, i) => i !== index));
  };

  const updateBonusMilestone = (index: number, field: "months" | "bonus_amount", value: number) => {
    const next = [...seniorityBonus];
    next[index] = { ...next[index], [field]: value };
    setSeniorityBonus(next);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      ...formData,
      rule_config: {
        frequency: formData.frequency,
        accrual_amount: formData.accrual_amount,
        accrual_timing: "start_of_period",
        carry_over_limit: formData.carry_over_limit,
        min_work_days_threshold: formData.min_work_days_threshold,
        seniority_bonus: seniorityBonus.sort((a, b) => a.months - b.months)
      }
    });
  };



  return (
    <Modal isOpen={isOpen} onClose={onClose} title={policy ? "Edit Policy" : "Create Leave Policy"} maxWidth="lg">
      <form onSubmit={handleSubmit} className="p-8 space-y-12 max-h-[85vh] overflow-y-auto custom-scrollbar">
        {/* IDENTITY */}
        <div className="space-y-8">
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-xl bg-primary-50 flex items-center justify-center text-primary-600"><LayoutGrid size={18} /></div>
            <h4 className="text-sm font-black text-slate-800 uppercase tracking-tighter">Identity</h4>
          </div>
          <div className="space-y-6 px-2">
            <FormRow label="Policy Name">
              <input required type="text" value={formData.name} onChange={(e) => setFormData({...formData, name: e.target.value})} className="w-full rounded-xl border border-slate-200 bg-white p-3 text-sm font-bold outline-none focus:ring-4 focus:ring-primary-500/5 transition-all h-[54px]" id="policy-name" placeholder="e.g. Standard Policy" />
            </FormRow>
            <FormRow label="Leave Type">
              <Combobox options={leaveTypes.map((t: any) => ({ value: t.id, label: t.name }))} value={formData.leave_type_id} onChange={(val) => setFormData({...formData, leave_type_id: val})} className="bg-white border-slate-200 h-[54px] rounded-xl" />
            </FormRow>
          </div>
        </div>

        {/* ACCRUAL RULES */}
        <div className="space-y-8">
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-xl bg-emerald-50 flex items-center justify-center text-emerald-600"><Clock size={18} /></div>
            <h4 className="text-sm font-black text-slate-800 uppercase tracking-tighter">Accrual Logic</h4>
          </div>
          <div className="bg-slate-50/50 p-10 rounded-[2.5rem] border border-slate-100 space-y-6">
            <FormRow label="Base Amount" info="Days to grant per period">
              <input required type="number" step="0.1" id="annual-entitlement" value={formData.accrual_amount} onChange={(e) => setFormData({...formData, accrual_amount: parseFloat(e.target.value)})} className="w-full rounded-xl border border-slate-200 bg-white p-3 text-sm font-bold font-mono text-center h-[54px]" />
            </FormRow>
            <FormRow label="Frequency">
              <select value={formData.frequency} onChange={(e) => setFormData({...formData, frequency: e.target.value})} className="w-full rounded-xl border border-slate-200 bg-white p-3 text-sm font-bold uppercase h-[54px]">
                <option value="monthly">Monthly</option>
                <option value="quarterly">Quarterly</option>
                <option value="yearly">Yearly</option>
              </select>
            </FormRow>
            <FormRow label="Min Days Work" info="Threshold to receive accrual">
              <input required type="number" step="0.5" value={formData.min_work_days_threshold} onChange={(e) => setFormData({...formData, min_work_days_threshold: parseFloat(e.target.value)})} className="w-full rounded-xl border border-slate-200 bg-white p-3 text-sm font-bold font-mono text-center text-amber-600 h-[54px]" />
            </FormRow>
            <FormRow label="Carry Limit" info="Max days to carry over next year">
              <input required type="number" value={formData.carry_over_limit} onChange={(e) => setFormData({...formData, carry_over_limit: parseInt(e.target.value)})} className="w-full rounded-xl border border-slate-200 bg-white p-3 text-sm font-bold font-mono text-center h-[54px]" />
            </FormRow>
          </div>
        </div>

        {/* SENIORITY BONUS */}
        <div className="space-y-8">
          <div className="flex justify-between items-center px-1">
            <div className="flex items-center gap-3">
              <div className="h-8 w-8 rounded-xl bg-indigo-50 flex items-center justify-center text-indigo-600"><Milestone size={18} /></div>
              <h4 className="text-sm font-black text-slate-800 uppercase tracking-tighter">Service Bonuses</h4>
            </div>
            <button type="button" onClick={addBonusMilestone} className="flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-indigo-700 transition-all shadow-lg shadow-indigo-100"><Plus size={12} /> Add Row</button>
          </div>
          
          <div className="space-y-8 px-2">
            {seniorityBonus.length === 0 ? (
              <div className="py-10 text-center bg-white border-2 border-dashed border-slate-100 rounded-[2rem] text-slate-400 italic text-xs">No bonus milestones defined.</div>
            ) : seniorityBonus.map((bonus, idx) => (
              <div key={idx} className="space-y-6 pt-8 first:pt-0 border-t first:border-0 border-slate-100 animate-in slide-in-from-right-2 duration-300 relative group/row">
                <div className="absolute -left-4 top-10 font-black text-[8px] text-slate-300 uppercase vertical-text tracking-widest">Row {idx + 1}</div>
                
                <FormRow label="After Months">
                  <input type="number" value={bonus.months} onChange={(e) => updateBonusMilestone(idx, "months", parseInt(e.target.value))} className="w-full rounded-xl border border-slate-200 bg-white p-3 text-sm font-bold font-mono text-center h-[54px] focus:ring-4 focus:ring-indigo-500/5 transition-all outline-none" />
                </FormRow>
                
                <FormRow label="Bonus Amount">
                  <div className="flex gap-4 w-full max-w-md">
                    <input type="number" step="0.5" value={bonus.bonus_amount} onChange={(e) => updateBonusMilestone(idx, "bonus_amount", parseFloat(e.target.value))} className="flex-1 rounded-xl border border-slate-200 bg-white p-3 text-sm font-bold font-mono text-center text-primary-600 h-[54px] focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" />
                    <button type="button" onClick={() => removeBonusMilestone(idx)} className="p-4 text-slate-300 hover:text-rose-600 hover:bg-rose-50 rounded-2xl transition-all h-[54px] flex items-center justify-center border border-transparent hover:border-rose-100 shadow-sm hover:shadow-md"><Trash2 size={18} /></button>
                  </div>
                </FormRow>
              </div>
            ))}
          </div>
        </div>

        {/* FOOTER */}
        <div className="flex justify-end gap-4 pt-8 border-t border-slate-100 sticky bottom-0 bg-white pb-2">
          <button type="button" onClick={onClose} className="px-8 py-3 text-xs font-bold text-slate-400 uppercase tracking-[0.2em] hover:text-slate-600">Cancel</button>
          <button type="submit" data-testid="save-policy-btn" disabled={isSaving} className="rounded-2xl bg-primary-600 px-10 py-4 text-xs font-black text-white uppercase tracking-[0.2em] shadow-2xl shadow-primary-200 hover:bg-primary-700 active:scale-95 transition-all">
            {isSaving ? "Saving..." : (policy ? "Update Policy" : "Create Policy")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
