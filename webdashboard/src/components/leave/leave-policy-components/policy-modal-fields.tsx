import { LayoutGrid, Clock, Milestone, Plus, Trash2 } from "lucide-react";
import React from "react";
 
import { Combobox } from "@/components/ui/combobox";
import { SeniorityBonus } from "@/lib/api/types";

interface PolicyFormData {
  accrual_amount: number;
  carry_over_limit: number;
  description: string;
  frequency: string;
  leave_type_id: string;
  min_work_days_threshold: number;
  name: string;
}

export const FormRow = ({ label, children, info }: { label: string; children: React.ReactNode; info?: string }) => (
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

export function PolicyIdentitySection({
  formData,
  setFormData,
  leaveTypes
}: {
  formData: PolicyFormData;
  leaveTypes: { id: string; name: string }[];
  setFormData: (data: PolicyFormData) => void;
}) {
  return (
    <div className="space-y-8">
      <div className="flex items-center gap-3">
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
  );
}

export function AccrualLogicSection({
  formData,
  setFormData
}: {
  formData: PolicyFormData;
  setFormData: (data: PolicyFormData) => void;
}) {
  return (
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
  );
}

export function SeniorityBonusSection({
  seniorityBonus,
  addBonusMilestone,
  updateBonusMilestone,
  removeBonusMilestone
}: {
  addBonusMilestone: () => void;
  removeBonusMilestone: (i: number) => void;
  seniorityBonus: SeniorityBonus[];
  updateBonusMilestone: (i: number, f: keyof SeniorityBonus, v: number) => void;
}) {
  return (
    <div className="space-y-8">
      <div className="flex justify-between items-center px-1">
        <div className="flex items-center gap-3">
          <div className="h-8 w-8 rounded-xl bg-blue-50 dark:bg-blue-950 flex items-center justify-center text-blue-600"><Milestone size={18} /></div>
          <h4 className="text-sm font-black text-slate-800 dark:text-slate-100 uppercase tracking-tighter">Service Bonuses</h4>
        </div>
        <button className="flex items-center gap-2 rounded-xl bg-primary-600 px-4 py-2 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100" onClick={addBonusMilestone} type="button"><Plus size={12} /> Add Row</button>
      </div>
      
      <div className="space-y-8 px-2">
        {seniorityBonus.length === 0 ? (
          <div className="py-10 text-center bg-white dark:bg-slate-900 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2rem] text-slate-400 dark:text-slate-500 italic text-xs">No bonus milestones defined.</div>
        ) : seniorityBonus.map((bonus, index) => (
          <div className="space-y-6 pt-8 first:pt-0 border-t first:border-0 border-slate-100 dark:border-slate-800 animate-in slide-in-from-right-2 duration-300 relative group/row" key={index}>
            <div className="absolute -left-4 top-10 font-black text-[8px] text-slate-300 dark:text-slate-600 uppercase vertical-text tracking-widest">Row {index + 1}</div>
            
            <FormRow label="After Years">
              <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-3 text-sm font-bold font-mono text-center h-[54px] focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" onChange={(e) => { updateBonusMilestone(index, "min_years", parseInt(e.target.value)); }} type="number" value={bonus.min_years} />
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
  );
}
