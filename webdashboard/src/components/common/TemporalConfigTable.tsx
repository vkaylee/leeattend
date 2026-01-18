import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Trash2, Calendar, History, Save } from "lucide-react";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

interface ConfigRecord {
  id?: string;
  std_working_days: number;
  hours_per_day: number;
  effective_date: string;
  created_at?: string;
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
  
  const firstOfMonth = new Date();
  firstOfMonth.setDate(1);
  const [newEffectiveDate, setNewEffectiveDate] = useState(firstOfMonth.toISOString().split('T')[0]);

  const handleSave = async () => {
    await onSave({ std_working_days: newDays, hours_per_day: newHours, effective_date: newEffectiveDate });
    setIsAdding(false);
  };

  // Sort records by effective_date DESC
  const sortedRecords = [...records].sort((a, b) => 
    new Date(b.effective_date).getTime() - new Date(a.effective_date).getTime()
  );

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-slate-400">
          <History size={16} />
          <h3 className="text-[10px] font-black uppercase tracking-widest">{title}</h3>
        </div>
        {!isAdding && (
          <button 
            onClick={() => setIsAdding(true)}
            className="flex items-center gap-1 text-[10px] font-black uppercase tracking-widest text-primary-600 hover:text-primary-700 transition-colors"
          >
            <Plus size={12} /> {t("common.add_new", "Thêm mới")}
          </button>
        )}
      </div>

      {isAdding && (
        <div className="bg-primary-50/50 border border-primary-100 rounded-2xl p-4 animate-in fade-in slide-in-from-top-2">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
            <div className="space-y-1">
              <label className="text-[9px] font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("payroll.settings.effective_date", "Ngày hiệu lực")}
              </label>
              <input 
                type="date" 
                value={newEffectiveDate}
                onChange={e => setNewEffectiveDate(e.target.value)}
                className="w-full bg-white border border-slate-200 rounded-xl px-3 py-2 text-sm font-bold focus:ring-2 focus:ring-primary-500 outline-none"
              />
            </div>
            <div className="space-y-1">
              <label className="text-[9px] font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("payroll.settings.std_days", "Ngày công chuẩn")}
              </label>
              <input 
                type="number" 
                value={newDays}
                onChange={e => setNewDays(parseFloat(e.target.value))}
                className="w-full bg-white border border-slate-200 rounded-xl px-3 py-2 text-sm font-bold focus:ring-2 focus:ring-primary-500 outline-none"
              />
            </div>
            <div className="space-y-1">
              <label className="text-[9px] font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("payroll.settings.hours_per_day", "Số giờ/ngày")}
              </label>
              <input 
                type="number" 
                value={newHours}
                onChange={e => setNewHours(parseFloat(e.target.value))}
                className="w-full bg-white border border-slate-200 rounded-xl px-3 py-2 text-sm font-bold focus:ring-2 focus:ring-primary-500 outline-none"
              />
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button 
              onClick={() => setIsAdding(false)}
              className="px-4 py-2 text-[10px] font-black uppercase tracking-widest text-slate-400 hover:text-slate-600 transition-colors"
            >
              {t("common.cancel")}
            </button>
            <button 
              onClick={handleSave}
              className="flex items-center gap-2 bg-primary-600 text-white px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100"
            >
              <Save size={12} /> {t("common.save", "Lưu")}
            </button>
          </div>
        </div>
      )}

      <div className="bg-slate-50/50 border border-slate-100 rounded-2xl overflow-hidden">
        {isLoading ? (
          <div className="p-4 text-center text-slate-400 text-xs italic">{t("common.loading")}</div>
        ) : sortedRecords.length === 0 ? (
          <div className="p-8 text-center">
            <p className="text-[10px] font-bold text-slate-300 uppercase tracking-widest">
              {t("payroll.settings.no_history", "Chưa có lịch sử cấu hình")}
            </p>
          </div>
        ) : (
          <table className="w-full text-left">
            <thead>
              <tr className="border-b border-slate-100 bg-slate-100/50">
                <th className="px-4 py-2 text-[9px] font-black text-slate-400 uppercase tracking-widest">{t("payroll.settings.effective_date", "Hiệu lực từ")}</th>
                <th className="px-4 py-2 text-[9px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.std_days", "Công chuẩn")}</th>
                <th className="px-4 py-2 text-[9px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.hours_per_day", "Giờ/Ngày")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {sortedRecords.map((record, idx) => (
                <tr key={idx} className="hover:bg-white transition-colors group">
                  <td className="px-4 py-2.5 text-xs font-bold text-slate-600 flex items-center gap-2">
                    <Calendar size={12} className="text-slate-300" />
                    {formatDate(record.effective_date)}
                    {idx === 0 && (
                      <span className="bg-emerald-100 text-emerald-700 text-[8px] font-black uppercase px-1.5 py-0.5 rounded-md">
                        {t("common.active", "Hiện tại")}
                      </span>
                    )}
                  </td>
                  <td className="px-4 py-2.5 text-xs font-black text-slate-900 text-right font-mono">
                    {record.std_working_days}
                  </td>
                  <td className="px-4 py-2.5 text-xs font-bold text-slate-500 text-right font-mono">
                    {record.hours_per_day}h
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
