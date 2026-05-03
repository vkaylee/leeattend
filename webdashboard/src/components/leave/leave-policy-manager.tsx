import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Clock, Pencil, ShieldCheck, Milestone, LayoutGrid } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { fetchLeavePolicies, createLeavePolicy, deleteLeavePolicy, updateLeavePolicy, fetchLeaveTypes } from "@/lib/api/leaves";
import { LeavePolicy } from "@/lib/api/types";
const FormRow = ({ label, children, info }: { label: string; children: React.ReactNode; info?: string }) => (
  <div className="grid grid-cols-1 md:grid-cols-12 gap-4 items-center group">
    <div className="md:col-span-4 md:text-right">
      <label className="text-[10px] font-black text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
        {label}
      </label>
    </div>
    <div className="md:col-span-8 flex flex-col">
      <div className="w-full max-w-md">
        {children}
      </div>
      {!!info && <p className="text-[9px] text-slate-400 dark:text-slate-500 italic font-medium ml-1 mt-1">{info}</p>}
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
  const { confirm, ConfirmationDialog } = useConfirm();

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
      void queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success("Policy deleted successfully");
    },
  });

  const handleDelete = async (id: string) => {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("leave.policy.confirm_delete", { defaultValue: "Delete this policy?" }),
      variant: "danger"
    });
    if (ok) deleteMutation.mutate(id);
  };

  const mutation = useMutation({
    mutationFn: (data: Partial<LeavePolicy>) => 
      editingPolicy ? updateLeavePolicy(wsId, editingPolicy.id, data) : createLeavePolicy(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-policies"] });
      toast.success(editingPolicy ? "Policy updated" : "Policy created");
      setIsCreating(false);
      setEditingPolicy(null);
    },
    onError: (error: Error) => { toast.error(error.message); },
  });

  const _calculateAnnualEntitlement = (policy: LeavePolicy) => {
    const amount = policy.rule_config.accrual_amount;
    switch (policy.rule_config.frequency) {
      case "monthly": return amount * 12;
      case "quarterly": return amount * 4;
      case "yearly": return amount;
      default: return amount;
    }
  };

  if (isLoading) return <div className="p-12 text-center text-slate-400 dark:text-slate-500 animate-pulse font-bold uppercase tracking-widest text-[10px]">Loading policies...</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-end px-1">
         <button className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100" data-testid="add-policy-btn" onClick={() => { setIsCreating(true); }}><Plus size={14} /> Create Policy</button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {policies.length === 0 ? (
          <div className="col-span-full py-20 bg-slate-50 dark:bg-slate-800 border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-[2.5rem] flex flex-col items-center justify-center text-slate-400 dark:text-slate-500">
            <ShieldCheck className="mb-4 opacity-20" size={40} />
            <p className="text-xs font-bold uppercase tracking-widest">No policies defined yet</p>
          </div>
        ) : policies.map((policy) => {
          const type = leaveTypes.find(t => t.id === policy.leave_type_id);
          const bonusCount = policy.rule_config.seniority_bonus.length || 0;
          return (
            <div className="relative flex flex-col justify-between rounded-[2rem] border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-7 shadow-sm hover:shadow-md transition-all group" key={policy.id}>
              <div>
                <div className="flex items-center justify-between">
                  <span className="inline-flex items-center rounded-lg bg-primary-50 dark:bg-primary-950 px-2.5 py-1 text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest border border-primary-100 dark:border-primary-800 shadow-sm">
                    {type?.name ?? "Unknown"}
                  </span>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button className="p-2 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-800" data-testid="edit-policy-btn" onClick={() => { setEditingPolicy(policy); }}><Pencil size={16} /></button>
                    <button className="p-2 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-800" data-testid="delete-policy-btn" onClick={() => handleDelete(policy.id)}><Trash2 size={16} /></button>
                  </div>
                </div>
                <h3 className="mt-5 text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight leading-none uppercase">{policy.name}</h3>
                
                <div className="mt-6 p-4 bg-slate-50 dark:bg-slate-800 rounded-2xl border border-slate-100 dark:border-slate-800 space-y-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-slate-400 dark:text-slate-500"><Clock size={14} /><span className="text-[10px] font-bold uppercase tracking-widest">Base Accrual</span></div>
                    <span className="text-[10px] font-bold text-slate-600 dark:text-slate-300 uppercase tracking-widest">{policy.rule_config.accrual_amount} / {policy.rule_config.frequency}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-slate-400 dark:text-slate-500"><Milestone size={14} /><span className="text-[10px] font-bold uppercase tracking-widest">Bonus Steps</span></div>
                    <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest">{bonusCount} steps</span>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {!!(isCreating || editingPolicy) && (
        <PolicyModal
          isOpen={true}
          isSaving={mutation.isPending}
          leaveTypes={leaveTypes}
          onClose={() => { setIsCreating(false); setEditingPolicy(null); }}
          onSave={(data: Partial<LeavePolicy>) => { mutation.mutate(data); }}
 
          policy={editingPolicy}
        />
      )}
      <ConfirmationDialog />
    </div>
  );
}

interface PolicyModalProps {
 
  isOpen: boolean;
  isSaving: boolean;
  leaveTypes: { id: string; name: string }[];
  onClose: () => void;
  onSave: (data: Partial<LeavePolicy>) => void;
  policy: LeavePolicy | null;
}

function PolicyModal({ isOpen, onClose, leaveTypes, policy, onSave, isSaving }: PolicyModalProps) {
  const [formData, setFormData] = useState({
    name: policy?.name ?? "",
    description: policy?.description ?? "",
    leave_type_id: policy?.leave_type_id ?? (leaveTypes.length > 0 ? leaveTypes[0].id : ""),
    accrual_amount: policy?.rule_config.accrual_amount ?? 1.0,
    frequency: (policy?.rule_config.frequency ?? "monthly"),
    carry_over_limit: policy?.rule_config.carry_over_limit ?? 5,
    min_work_days_threshold: policy?.rule_config.min_work_days_threshold ?? 14
  });

  const [seniorityBonus, setSeniorityBonus] = useState<{ min_years: number; extra_days: number }[]>(
    policy?.rule_config.seniority_bonus ?? []
  );

  const addBonusMilestone = () => {
    setSeniorityBonus([...seniorityBonus, { min_years: 1, extra_days: 1.0 }]);
  };

  const removeBonusMilestone = (index: number) => {
    setSeniorityBonus(seniorityBonus.filter((_, index_) => index_ !== index));
  };

  const updateBonusMilestone = (index: number, field: "min_years" | "extra_days", value: number) => {
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
        seniority_bonus: seniorityBonus.sort((a, b) => a.min_years - b.min_years)
      }
    });
  };



  return (
    <Modal isOpen={isOpen} maxWidth="lg" onClose={onClose} title={policy ? "Edit Policy" : "Create Leave Policy"}>
      <form className="p-8 space-y-12 max-h-[85vh] overflow-y-auto custom-scrollbar" onSubmit={handleSubmit}>
        {/* IDENTITY */}
        <div className="space-y-8">
          <div className="flex items-center gap-3">
            { }
            <div className="h-8 w-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400"><LayoutGrid size={18} /></div>
            <h4 className="text-sm font-black text-slate-800 dark:text-slate-100 uppercase tracking-tighter">Identity</h4>
          </div>
          <div className="space-y-6 px-2">
            <FormRow label="Policy Name">
              <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold outline-none focus:ring-4 focus:ring-primary-500/5 transition-all h-[54px]" id="policy-name" onChange={(e) => { setFormData({...formData, name: e.target.value}); }} placeholder="e.g. Standard Policy" required type="text" value={formData.name} />
            </FormRow>
            <FormRow label="Leave Type">
              <Combobox className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-[54px] rounded-xl" onChange={(value) => { setFormData({...formData, leave_type_id: value}); }} options={leaveTypes.map((t) => ({ value: t.id, label: t.name }))} value={formData.leave_type_id} />
            </FormRow>
          </div>
        </div>

        {/* ACCRUAL RULES */}
        <div className="space-y-8">
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-xl bg-emerald-50 dark:bg-emerald-950 flex items-center justify-center text-emerald-600 dark:text-emerald-300"><Clock size={18} /></div>
            <h4 className="text-sm font-black text-slate-800 dark:text-slate-100 uppercase tracking-tighter">Accrual Logic</h4>
          </div>
          <div className="bg-slate-50 dark:bg-slate-800/50 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 space-y-6">
            <FormRow info="Days to grant per period" label="Base Amount">
              <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold font-mono text-center h-[54px]" id="annual-entitlement" onChange={(e) => { setFormData({...formData, accrual_amount: parseFloat(e.target.value)}); }} required step="0.1" type="number" value={formData.accrual_amount} />
            </FormRow>
            <FormRow label="Frequency">
              <select className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold uppercase h-[54px]" onChange={(e) => { setFormData({...formData, frequency: e.target.value as "monthly" | "quarterly" | "yearly"}); }} value={formData.frequency}>
                <option value="monthly">Monthly</option>
                <option value="quarterly">Quarterly</option>
                <option value="yearly">Yearly</option>
              </select>
            </FormRow>
            <FormRow info="Threshold to receive accrual" label="Min Days Work">
              <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold font-mono text-center text-amber-600 dark:text-amber-300 h-[54px]" onChange={(e) => { setFormData({...formData, min_work_days_threshold: parseFloat(e.target.value)}); }} required step="0.5" type="number" value={formData.min_work_days_threshold} />
            </FormRow>
            <FormRow info="Max days to carry over next year" label="Carry Limit">
              <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold font-mono text-center h-[54px]" onChange={(e) => { setFormData({...formData, carry_over_limit: parseInt(e.target.value)}); }} required type="number" value={formData.carry_over_limit} />
            </FormRow>
          </div>
        </div>

        {/* SENIORITY BONUS */}
        <div className="space-y-8">
          <div className="flex justify-between items-center px-1">
            <div className="flex items-center gap-3">
              <div className="h-8 w-8 rounded-xl bg-indigo-50 dark:bg-indigo-950 flex items-center justify-center text-indigo-600"><Milestone size={18} /></div>
              <h4 className="text-sm font-black text-slate-800 dark:text-slate-100 uppercase tracking-tighter">Service Bonuses</h4>
            </div>
            <button className="flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-indigo-700 transition-all shadow-lg shadow-indigo-100" onClick={addBonusMilestone} type="button"><Plus size={12} /> Add Row</button>
          </div>
          
          <div className="space-y-8 px-2">
            {seniorityBonus.length === 0 ? (
              <div className="py-10 text-center bg-white dark:bg-slate-900 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2rem] text-slate-400 dark:text-slate-500 italic text-xs">No bonus milestones defined.</div>
            ) : seniorityBonus.map((bonus, index) => (
              <div className="space-y-6 pt-8 first:pt-0 border-t first:border-0 border-slate-100 dark:border-slate-800 animate-in slide-in-from-right-2 duration-300 relative group/row" key={index}>
                <div className="absolute -left-4 top-10 font-black text-[8px] text-slate-300 dark:text-slate-600 uppercase vertical-text tracking-widest">Row {index + 1}</div>
                
                <FormRow label="After Years">
                  <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold font-mono text-center h-[54px] focus:ring-4 focus:ring-indigo-500/5 transition-all outline-none" onChange={(e) => { updateBonusMilestone(index, "min_years", parseInt(e.target.value)); }} type="number" value={bonus.min_years} />
                </FormRow>
                
                <FormRow label="Bonus Amount">
                  <div className="flex gap-4 w-full max-w-md">
                    <input className="flex-1 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold font-mono text-center text-primary-600 dark:text-primary-400 h-[54px] focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" onChange={(e) => { updateBonusMilestone(index, "extra_days", parseFloat(e.target.value)); }} step="0.5" type="number" value={bonus.extra_days} />
                    <button className="p-4 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 rounded-2xl transition-all h-[54px] flex items-center justify-center border border-transparent hover:border-rose-100 dark:border-rose-800 shadow-sm hover:shadow-md" onClick={() => { removeBonusMilestone(index); }} type="button"><Trash2 size={18} /></button>
                  </div>
                </FormRow>
              </div>
            ))}
          </div>
        </div>

        {/* FOOTER */}
        <div className="flex justify-end gap-4 pt-8 border-t border-slate-100 dark:border-slate-800 sticky bottom-0 bg-white dark:bg-slate-900 pb-2">
          <button className="px-8 py-3 text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600" onClick={onClose} type="button">Cancel</button>
          <button className="rounded-2xl bg-primary-600 px-10 py-4 text-xs font-black text-white uppercase tracking-[0.2em] shadow-2xl shadow-primary-200 hover:bg-primary-700 active:scale-95 transition-all" data-testid="save-policy-btn" disabled={isSaving} type="submit">
            {isSaving ? "Saving..." : (policy ? "Update Policy" : "Create Policy")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
