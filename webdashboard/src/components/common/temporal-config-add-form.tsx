import type { TFunction } from "i18next";

import { AlertCircle, Plus, Save } from "lucide-react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

export interface TemporalConfigAddFormProps {
  handleSave: () => Promise<void>;
  newDays: number;
  newEffectiveDate: string;
  newHours: number;
  newMaxOtDay: string;
  newMaxOtMonth: string;
  setIsAdding: (val: boolean) => void;
  setNewDays: (val: number) => void;
  setNewEffectiveDate: (val: string) => void;
  setNewHours: (val: number) => void;
  setNewMaxOtDay: (val: string) => void;
  setNewMaxOtMonth: (val: string) => void;
  t: TFunction;
}

export function TemporalConfigAddForm({
  newEffectiveDate,
  setNewEffectiveDate,
  newDays,
  setNewDays,
  newHours,
  setNewHours,
  newMaxOtDay,
  setNewMaxOtDay,
  newMaxOtMonth,
  setNewMaxOtMonth,
  setIsAdding,
  handleSave,
  t,
}: TemporalConfigAddFormProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 rounded-[2.5rem] p-4 sm:p-8 shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500">
      <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
        <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
        <Typography className="text-[10px] font-bold uppercase tracking-[0.2em]" variant="h4">{t("payroll.settings.add_version")}</Typography>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="payroll-effective-date">{t("payroll.settings.effective_date")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all" id="payroll-effective-date" onChange={e => { setNewEffectiveDate(e.target.value); }} type="date" value={newEffectiveDate} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="payroll-std-days">{t("payroll.settings.std_days")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" id="payroll-std-days" onChange={e => { setNewDays(parseFloat(e.target.value)); }} type="number" value={Number.isNaN(newDays) ? "" : newDays} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="payroll-hours-per-day">{t("payroll.settings.hours_per_day")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" id="payroll-hours-per-day" onChange={e => { setNewHours(parseFloat(e.target.value)); }} type="number" value={Number.isNaN(newHours) ? "" : newHours} />
        </div>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="payroll-max-ot-day">{t("payroll.settings.max_ot_hours_per_day")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" id="payroll-max-ot-day" onChange={e => { setNewMaxOtDay(e.target.value); }} placeholder={t("payroll.settings.no_limit") || ""} step="0.5" type="number" value={newMaxOtDay} />
          <Typography className="text-[9px] text-slate-400 ml-1" variant="p">{t("payroll.settings.ot_empty_hint")}</Typography>
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="payroll-max-ot-month">{t("payroll.settings.max_ot_hours_per_month")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" id="payroll-max-ot-month" onChange={e => { setNewMaxOtMonth(e.target.value); }} placeholder={t("payroll.settings.no_limit") || ""} step="0.5" type="number" value={newMaxOtMonth} />
          <Typography className="text-[9px] text-slate-400 ml-1" variant="p">{t("payroll.settings.ot_empty_hint")}</Typography>
        </div>
      </div>

      <div className="p-6 bg-amber-50 dark:bg-amber-950/50 border border-amber-100 dark:border-amber-800 rounded-[1.5rem] flex gap-4 mb-8 shadow-inner">
        <AlertCircle className="w-6 h-6 text-amber-500 shrink-0" />
        <Typography className="text-[10px] text-amber-800 leading-relaxed font-bold uppercase tracking-tight opacity-80" variant="p">{t("payroll.settings.date_warning")}</Typography>
      </div>

      {!Number.isFinite(newDays) || newDays <= 0 ? (
        <Typography className="mb-4 text-sm text-red-700" role="alert" variant="p">{t("payroll.settings.validation.positive")}</Typography>
      ) : null}
      {!Number.isFinite(newHours) || newHours <= 0 ? (
        <Typography className="mb-4 text-sm text-red-700" role="alert" variant="p">{t("payroll.settings.validation.positive")}</Typography>
      ) : null}

      <div className="flex justify-end gap-4">
        <Button className="px-8 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={() => { setIsAdding(false); }} variant="ghost">{t("common.cancel")}</Button>
        <Button className="flex items-center gap-3 bg-primary-600 text-white px-10 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" onClick={handleSave} variant="primary"><Save size={16} /> {t("common.save").toUpperCase()}</Button>
      </div>
    </div>
  );
}
