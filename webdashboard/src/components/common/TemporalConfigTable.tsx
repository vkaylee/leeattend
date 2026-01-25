import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Trash2, Calendar, History, Save, Calculator, AlertCircle, CheckCircle2 } from "lucide-react";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { cn } from "@/lib/utils";

interface ConfigRecord {
  id?: string;
  std_working_days: number;
  hours_per_day: number;
  effective_date: string;
  created_at?: string;
  is_virtual?: boolean;
}

interface Props {
  title: string;
  records: ConfigRecord[];
  onSave: (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => Promise<void>;
  isLoading?: boolean;
}

export default function TemporalConfigTable({ title, records, onSave, isLoading }: Props) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();
  const [isAdding, setIsAdding] = useState(false);
  const [newDays, setNewDays] = useState<number>(26);
  const [newHours, setNewHours] = useState<number>(8);
  
  const firstOfMonth = new Date(); firstOfMonth.setDate(1);
  const [newEffectiveDate, setNewEffectiveDate] = useState(firstOfMonth.toISOString().split('T')[0]);

  const handleSave = async () => {
    await onSave({ std_working_days: newDays, hours_per_day: newHours, effective_date: newEffectiveDate });
    setIsAdding(false);
  };

  const displayRecords = [...records];
  if (displayRecords.length === 0 && !isLoading) {
    displayRecords.push({ std_working_days: 26, hours_per_day: 8, effective_date: "1970-01-01", is_virtual: true });
  }

  const sortedRecords = displayRecords.sort((a, b) => new Date(b.effective_date).getTime() - new Date(a.effective_date).getTime());
  const now = new Date();
  const activeIdx = sortedRecords.findIndex(r => new Date(r.effective_date) <= now);

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><Calculator size={20} /></div>
          <div>
            <h3 className="text-lg font-bold text-slate-800 tracking-tight uppercase">{title}</h3>
            <p className="text-[10px] font-bold text-slate-500 uppercase tracking-widest mt-0.5">{t("payroll.settings.policy_desc")}</p>
          </div>
        </div>
        {!isAdding && (
          <button onClick={() => setIsAdding(true)} className="flex items-center justify-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95"><Plus size={14} /> {t("common.add_new").toUpperCase()}</button>
        )}
      </div>

      {isAdding && (
        <div className="bg-white border-2 border-primary-100 rounded-[2.5rem] p-8 shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500">
          <div className="flex items-center gap-3 mb-8 text-primary-700">
            <div className="w-8 h-8 rounded-xl bg-primary-50 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
            <h4 className="text-[10px] font-bold uppercase tracking-[0.2em]">{t("payroll.settings.add_version")}</h4>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("payroll.settings.effective_date")}</label>
              <input type="date" value={newEffectiveDate} onChange={e => setNewEffectiveDate(e.target.value)} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all" />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("payroll.settings.std_days")}</label>
              <input type="number" value={newDays} onChange={e => setNewDays(parseFloat(e.target.value))} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono" />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("payroll.settings.hours_per_day")}</label>
              <input type="number" value={newHours} onChange={e => setNewHours(parseFloat(e.target.value))} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono" />
            </div>
          </div>
          
          <div className="p-6 bg-amber-50/50 border border-amber-100 rounded-[1.5rem] flex gap-4 mb-8 shadow-inner">
            <AlertCircle className="w-6 h-6 text-amber-500 shrink-0" />
            <p className="text-[10px] text-amber-800 leading-relaxed font-bold uppercase tracking-tight opacity-80">{t("payroll.settings.date_warning")}</p>
          </div>

          <div className="flex justify-end gap-4">
            <button onClick={() => setIsAdding(false)} className="px-8 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button onClick={handleSave} className="flex items-center gap-3 bg-primary-600 text-white px-10 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95"><Save size={16} /> {t("common.save").toUpperCase()}</button>
          </div>
        </div>
      )}

      <div className="bg-white border border-slate-100 rounded-[2.5rem] overflow-hidden shadow-sm">
        {isLoading ? (
          <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : (
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("payroll.settings.status")}</th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("payroll.settings.effective_date")}</th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-right">{t("payroll.settings.std_days")}</th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-right">{t("payroll.settings.hours_per_day")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {sortedRecords.map((record, idx) => {
                const isActive = idx === activeIdx; const isFuture = new Date(record.effective_date) > now;
                return (
                  <tr key={idx} className={cn("group transition-all hover:bg-slate-50/50", isActive && "bg-emerald-50/30")}>
                    <td className="px-8 py-6">
                      {record.is_virtual ? (
                        <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-amber-50 text-amber-600 border-amber-100 shadow-sm"><AlertCircle size={12} /> {t("payroll.settings.virtual")}</span>
                      ) : isActive ? (
                        <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-emerald-50 text-emerald-600 border-emerald-100 shadow-sm"><CheckCircle2 size={12} /> {t("common.active")}</span>
                      ) : isFuture ? (
                        <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-blue-50 text-blue-600 border-blue-100 shadow-sm"><Calendar size={12} /> {t("common.upcoming")}</span>
                      ) : (
                        <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-slate-50 text-slate-500 border-slate-100 opacity-60"><History size={12} /> {t("common.expired")}</span>
                      )}
                    </td>
                    <td className="px-8 py-6"><div className={cn("text-sm font-bold uppercase tracking-tight", isActive ? "text-slate-700" : "text-slate-500")}>{record.is_virtual ? t("common.from_start") : formatDate(record.effective_date)}</div></td>
                    <td className="px-8 py-6 text-right font-mono font-bold"><span className={cn("text-sm", isActive ? "text-primary-600" : "text-slate-500")}>{record.std_working_days} {t("common.days_short")}</span></td>
                    <td className="px-8 py-6 text-right font-mono font-bold"><span className={cn("text-sm", isActive ? "text-slate-700" : "text-slate-300")}>{record.hours_per_day}h / {t("common.day_short")}</span></td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
