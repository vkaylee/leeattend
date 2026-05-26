import { LayoutGrid, Clock, Milestone, Plus, Trash2 } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";
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
  <div className="flex flex-col sm:flex-row sm:items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800 last:border-0 gap-4 group">
    <div className="flex-1">
      <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block group-hover:text-primary-600 transition-colors">
        {label}
      </label>
      {!!info && <Typography className="text-[10px] text-slate-400 dark:text-slate-500 mt-0.5 leading-relaxed" variant="p">{info}</Typography>}
    </div>
    <div className="w-full sm:w-[55%]">
      {children}
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
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <div className="h-8 w-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shrink-0"><LayoutGrid size={16} /></div>
        <Typography className="text-sm font-black text-slate-800 dark:text-slate-100 uppercase tracking-tighter" variant="h4">Identity</Typography>
      </div>
      <div className="rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 overflow-hidden">
        <FormRow label="Policy Name">
          <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 text-sm font-bold outline-none focus:ring-4 focus:ring-primary-500/5 transition-all h-11" id="policy-name" onChange={(e) => { setFormData({...formData, name: e.target.value}); }} placeholder="e.g. Standard Policy" required type="text" value={formData.name} />
        </FormRow>
        <FormRow label="Leave Type">
          <Combobox className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-11 rounded-xl" onChange={(value) => { setFormData({...formData, leave_type_id: value}); }} options={leaveTypes.map((t) => ({ value: t.id, label: t.name }))} value={formData.leave_type_id} />
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
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <div className="h-8 w-8 rounded-xl bg-emerald-50 dark:bg-emerald-950 flex items-center justify-center text-emerald-600 dark:text-emerald-300 shrink-0"><Clock size={16} /></div>
        <Typography className="text-sm font-black text-slate-800 dark:text-slate-100 uppercase tracking-tighter" variant="h4">Accrual Logic</Typography>
      </div>
      <div className="rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 overflow-hidden">
        <FormRow info="Days to grant per period" label="Base Amount">
          <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 text-sm font-bold font-mono text-center h-11 focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" id="annual-entitlement" onChange={(e) => { setFormData({...formData, accrual_amount: parseFloat(e.target.value)}); }} required step="0.1" type="number" value={formData.accrual_amount} />
        </FormRow>
        <FormRow label="Frequency">
          <select className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 text-sm font-bold uppercase h-11 focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" onChange={(e) => { setFormData({...formData, frequency: e.target.value}); }} value={formData.frequency}>
            <option value="monthly">Monthly</option>
            <option value="quarterly">Quarterly</option>
            <option value="yearly">Yearly</option>
          </select>
        </FormRow>
        <FormRow info="Threshold to receive accrual" label="Min Days Work">
          <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 text-sm font-bold font-mono text-center text-amber-600 dark:text-amber-300 h-11 focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" onChange={(e) => { setFormData({...formData, min_work_days_threshold: parseFloat(e.target.value)}); }} required step="0.5" type="number" value={formData.min_work_days_threshold} />
        </FormRow>
        <FormRow info="Max days to carry over next year" label="Carry Limit">
          <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 text-sm font-bold font-mono text-center h-11 focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" onChange={(e) => { setFormData({...formData, carry_over_limit: parseInt(e.target.value)}); }} required type="number" value={formData.carry_over_limit} />
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
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-3">
          <div className="h-8 w-8 rounded-xl bg-blue-50 dark:bg-blue-950 flex items-center justify-center text-blue-600 shrink-0"><Milestone size={16} /></div>
          <Typography className="text-sm font-black text-slate-800 dark:text-slate-100 uppercase tracking-tighter" variant="h4">Service Bonuses</Typography>
        </div>
        <Button
          className="shadow-primary-100 text-[10px]"
          onClick={addBonusMilestone}
          size="sm"
          type="button"
          variant="primary"
        >
          <Plus size={12} /> Add Row
        </Button>
      </div>
      
      {seniorityBonus.length === 0 ? (
        <div className="py-8 text-center bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 rounded-[2rem] text-slate-400 dark:text-slate-500 italic text-xs">No bonus milestones defined.</div>
      ) : (
        <div className="rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 overflow-hidden">
          {seniorityBonus.map((bonus, index) => (
            <div className="border-b border-slate-100 dark:border-slate-800 last:border-0 relative animate-in slide-in-from-right-2 duration-300" key={index}>
              <FormRow label={`Milestone ${index + 1}: After Years`}>
                <input className="w-full rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 text-sm font-bold font-mono text-center h-11 focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" onChange={(e) => { updateBonusMilestone(index, "min_years", parseInt(e.target.value)); }} type="number" value={bonus.min_years} />
              </FormRow>
              <FormRow label="Bonus Amount">
                <div className="flex gap-2 w-full">
                  <input className="flex-1 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 text-sm font-bold font-mono text-center text-primary-600 dark:text-primary-400 h-11 focus:ring-4 focus:ring-primary-500/5 transition-all outline-none" onChange={(e) => { updateBonusMilestone(index, "extra_days", parseFloat(e.target.value)); }} step="0.5" type="number" value={bonus.extra_days} />
                  <Button
                    className="h-11 px-3 text-slate-300 hover:text-rose-600 dark:text-slate-600 dark:hover:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 border-slate-200 hover:border-rose-200 dark:border-slate-700 dark:hover:border-rose-800"
                    onClick={() => { removeBonusMilestone(index); }}
                    size="icon"
                    type="button"
                    variant="outline"
                  >
                    <Trash2 size={16} />
                  </Button>
                </div>
              </FormRow>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
