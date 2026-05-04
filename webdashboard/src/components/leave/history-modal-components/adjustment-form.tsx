import { Plus, Minus, Calendar, MessageSquare, Loader2 } from "lucide-react";
import React from "react";

import { cn } from "@/lib/utils";

interface AdjustmentFormState {
  amount: string;
  date: string;
  leave_type_id: string;
  reason: string;
  type: string;
}

interface AdjustmentFormProps {
  adjustForm: AdjustmentFormState;
  isPending: boolean;
  leaveTypes: { id: string; name: string }[];
  onCancel: () => void;
  onSubmit: (e: React.SyntheticEvent) => void;
  setAdjustForm: (data: AdjustmentFormState) => void;
  t: (key: string) => string;
}

function Shield({ className, size = 24 }: { className?: string; size?: number }) {
  return (
    <svg className={className} fill="none" height={size} stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width={size} xmlns="http://www.w3.org/2000/svg">
      <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" />
    </svg>
  );
}

export function AdjustmentForm({ 
  adjustForm, 
  setAdjustForm, 
  leaveTypes, 
  onSubmit, 
  isPending, 
  onCancel, 
  t 
}: AdjustmentFormProps) {
  return (
    <form className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pt-2" onSubmit={onSubmit}>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.action_label")}</label>
          <div className="grid grid-cols-2 gap-3 p-1.5 bg-slate-100 dark:bg-slate-700/50 rounded-2xl border border-slate-200 dark:border-slate-700/50 shadow-inner">
            <button className={cn("flex items-center justify-center gap-2 py-3 rounded-xl text-xs font-bold uppercase tracking-widest transition-all", adjustForm.type === "add" ? "bg-white dark:bg-slate-900 text-emerald-600 dark:text-emerald-300 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600")} onClick={() => { setAdjustForm({ ...adjustForm, type: "add" }); }} type="button"><Plus size={14} /> {t("leave_balances.history.add")}</button>
            <button className={cn("flex items-center justify-center gap-2 py-3 rounded-xl text-xs font-bold uppercase tracking-widest transition-all", adjustForm.type === "deduct" ? "bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600")} onClick={() => { setAdjustForm({ ...adjustForm, type: "deduct" }); }} type="button"><Minus size={14} /> {t("leave_balances.history.deduct")}</button>
          </div>
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_requests.policy.table.type")}</label>
          <select className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" onChange={(e) => { setAdjustForm({ ...adjustForm, leave_type_id: e.target.value }); }} value={adjustForm.leave_type_id}>
            <option value="">{t("common.select")}</option>
            {leaveTypes.map(lt => <option key={lt.id} value={lt.id}>{lt.name}</option>)}
          </select>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.amount_placeholder").split('.')[0]}</label>
          <input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm font-mono" min="0.5" onChange={(e) => { setAdjustForm({ ...adjustForm, amount: e.target.value }); }} placeholder={t("leave_balances.history.amount_placeholder")} required step="0.5" type="number" value={adjustForm.amount} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.date")}</label>
          <div className="relative group">
            <input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm" onChange={(e) => { setAdjustForm({ ...adjustForm, date: e.target.value }); }} required type="date" value={adjustForm.date} />
            <Calendar className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
          </div>
        </div>
      </div>

      <div className="space-y-3">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("leave_balances.history.reason_label")}</label>
        <div className="relative group">
          <textarea className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none font-medium text-sm transition-all resize-none shadow-sm" onChange={(e) => { setAdjustForm({ ...adjustForm, reason: e.target.value }); }} placeholder={t("leave_balances.history.reason_placeholder")} required rows={3} value={adjustForm.reason} />
          <MessageSquare className="absolute right-4 top-4 w-4 h-4 text-slate-200 dark:text-slate-700 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
        </div>
      </div>

      <div className="flex gap-4 pt-4 border-t border-slate-50 dark:border-slate-800">
        <button className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={onCancel} type="button">{t("common.cancel")}</button>
        <button className="flex-1 bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-3" disabled={isPending} type="submit">
          {isPending ? <Loader2 className="animate-spin w-5 h-5" /> : <Shield size={18} />} {t("leave_balances.history.confirm_btn")}
        </button>
      </div>
    </form>
  );
}
