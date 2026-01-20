import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Trash2, Calendar, History, Save, Calculator, AlertCircle, CheckCircle2 } from "lucide-react";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

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
  
  const firstOfMonth = new Date();
  firstOfMonth.setDate(1);
  const [newEffectiveDate, setNewEffectiveDate] = useState(firstOfMonth.toISOString().split('T')[0]);

  const handleSave = async () => {
    await onSave({ std_working_days: newDays, hours_per_day: newHours, effective_date: newEffectiveDate });
    setIsAdding(false);
  };

  // Logic to determine records to display
  let displayRecords = [...records];
  
  // If empty, inject a virtual default record
  if (displayRecords.length === 0 && !isLoading) {
    displayRecords.push({
      std_working_days: 26,
      hours_per_day: 8,
      effective_date: "1970-01-01", // Long ago
      is_virtual: true
    });
  }

  // Sort records by effective_date DESC
  const sortedRecords = displayRecords.sort((a, b) => 
    new Date(b.effective_date).getTime() - new Date(a.effective_date).getTime()
  );

  // Find the currently active record (latest record where effective_date <= now)
  const now = new Date();
  const activeIdx = sortedRecords.findIndex(r => new Date(r.effective_date) <= now);

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <Calculator size={20} />
          </div>
          <div>
            <h3 className="text-lg font-bold text-slate-800 tracking-tight">{title}</h3>
            <p className="text-xs text-slate-500 font-medium">
              {t("payroll.settings.policy_desc", "Manage standard work units and their evolution over time.")}
            </p>
          </div>
        </div>
        {!isAdding && (
          <button 
            onClick={() => setIsAdding(true)}
            className="flex items-center justify-center gap-2 bg-primary-600 hover:bg-primary-700 text-white px-6 py-2.5 rounded-2xl font-black text-[10px] uppercase tracking-widest transition-all shadow-lg shadow-primary-100 active:scale-95"
          >
            <Plus size={14} /> {t("common.add_new", "Thêm mới")}
          </button>
        )}
      </div>

      {isAdding && (
        <div className="bg-white border-2 border-primary-100 rounded-3xl p-6 shadow-xl shadow-primary-50/50 animate-in fade-in slide-in-from-top-4 duration-300">
          <div className="flex items-center gap-2 mb-6 text-primary-700">
            <Plus className="w-4 h-4" />
            <h4 className="text-xs font-black uppercase tracking-widest">{t("payroll.settings.add_version", "Tạo phiên bản chính sách mới")}</h4>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("payroll.settings.effective_date", "Ngày hiệu lực")}
              </label>
              <input 
                type="date" 
                value={newEffectiveDate}
                onChange={e => setNewEffectiveDate(e.target.value)}
                className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 text-sm font-bold focus:ring-2 focus:ring-primary-500 outline-none transition-all"
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("payroll.settings.std_days", "Ngày công chuẩn")}
              </label>
              <input 
                type="number" 
                value={newDays}
                onChange={e => setNewDays(parseFloat(e.target.value))}
                className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 text-sm font-bold focus:ring-2 focus:ring-primary-500 outline-none transition-all"
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("payroll.settings.hours_per_day", "Số giờ/ngày")}
              </label>
              <input 
                type="number" 
                value={newHours}
                onChange={e => setNewHours(parseFloat(e.target.value))}
                className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 text-sm font-bold focus:ring-2 focus:ring-primary-500 outline-none transition-all"
              />
            </div>
          </div>
          
          <div className="p-4 bg-amber-50 border border-amber-100 rounded-2xl flex gap-3 mb-6">
            <AlertCircle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
            <div className="text-[10px] text-amber-800 leading-relaxed font-medium">
              {t("payroll.settings.date_warning", "Lưu ý: Ngày hiệu lực sẽ quyết định kỳ lương nào bị ảnh hưởng. Các thay đổi sẽ tự động áp dụng cho mọi phép tính lương từ ngày này trở đi.")}
            </div>
          </div>

          <div className="flex justify-end gap-3">
            <button 
              onClick={() => setIsAdding(false)}
              className="px-6 py-3 rounded-2xl text-[10px] font-black uppercase tracking-widest text-slate-400 hover:bg-slate-50 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button 
              onClick={handleSave}
              className="flex items-center gap-2 bg-primary-600 text-white px-8 py-3 rounded-2xl text-[10px] font-black uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100"
            >
              <Save size={14} /> {t("common.save", "Lưu phiên bản")}
            </button>
          </div>
        </div>
      )}

      <div className="bg-white border border-slate-100 rounded-[32px] overflow-hidden shadow-sm">
        {isLoading ? (
          <div className="p-12 text-center flex flex-col items-center gap-3">
            <div className="w-8 h-8 border-4 border-primary-100 border-t-primary-600 rounded-full animate-spin"></div>
            <p className="text-xs text-slate-400 font-bold italic">{t("common.loading")}</p>
          </div>
        ) : (
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("payroll.settings.status", "Trạng thái")}</th>
                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("payroll.settings.effective_date", "Hiệu lực từ")}</th>
                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.std_days", "Ngày công chuẩn")}</th>
                <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.hours_per_day", "Số giờ/ngày")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {sortedRecords.map((record, idx) => {
                const isActive = idx === activeIdx;
                const isFuture = new Date(record.effective_date) > now;

                return (
                  <tr 
                    key={idx} 
                    className={`group transition-all ${
                      isActive 
                        ? "bg-emerald-50/30 hover:bg-emerald-50/50" 
                        : "hover:bg-slate-50/50"
                    }`}
                  >
                    <td className="px-6 py-5">
                      {record.is_virtual ? (
                        <div className="flex items-center gap-2 text-amber-600">
                          <AlertCircle size={14} />
                          <span className="text-[10px] font-black uppercase tracking-tighter">{t("payroll.settings.virtual", "Mặc định hệ thống")}</span>
                        </div>
                      ) : isActive ? (
                        <div className="flex items-center gap-2 text-emerald-600">
                          <CheckCircle2 size={14} />
                          <span className="text-[10px] font-black uppercase tracking-tighter">{t("common.active", "Đang áp dụng")}</span>
                        </div>
                      ) : isFuture ? (
                        <div className="flex items-center gap-2 text-blue-500">
                          <Calendar size={14} />
                          <span className="text-[10px] font-black uppercase tracking-tighter">{t("common.upcoming", "Sắp tới")}</span>
                        </div>
                      ) : (
                        <div className="flex items-center gap-2 text-slate-300">
                          <History size={14} />
                          <span className="text-[10px] font-black uppercase tracking-tighter">{t("common.expired", "Đã hết hạn")}</span>
                        </div>
                      )}
                    </td>
                    <td className="px-6 py-5">
                      <div className={`text-sm font-bold ${isActive ? "text-slate-900" : "text-slate-500"}`}>
                        {record.is_virtual ? t("common.from_start", "Từ ngày bắt đầu") : formatDate(record.effective_date)}
                      </div>
                    </td>
                    <td className="px-6 py-5 text-right">
                      <span className={`text-sm font-black ${isActive ? "text-primary-600" : "text-slate-600"} font-mono`}>
                        {record.std_working_days} {t("common.days_short", "d")}
                      </span>
                    </td>
                    <td className="px-6 py-5 text-right">
                      <span className={`text-sm font-bold ${isActive ? "text-slate-900" : "text-slate-400"} font-mono`}>
                        {record.hours_per_day}h/{t("common.day_short", "d")}
                      </span>
                    </td>
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
