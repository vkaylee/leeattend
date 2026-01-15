import React from "react";
import { useTranslation } from "react-i18next";
import { ChevronRight, Clock, Zap, Save, Trash2 } from "lucide-react";
import { Shift } from "../../lib/api";
import { useDateTimeFormatter } from "../../hooks/use-datetime";

interface ShiftListProps {
  shifts: Shift[] | undefined;
  onEdit: (shift: Shift) => void;
  onDelete?: (shift: Shift) => void;
}

export function ShiftList({ shifts, onEdit, onDelete }: ShiftListProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  return (
    <div className="bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
        <h2 className="font-black text-[10px] text-slate-400 uppercase tracking-widest">{t("shifts.directory_title")}</h2>
        <span className="text-[10px] font-bold text-slate-400 italic">{t("common.total")}: {shifts?.length || 0}</span>
      </div>
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-slate-50/20">
            <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">{t("shifts.table.shift_name")}</th>
            <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">{t("shifts.table.time_range")}</th>
            <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100">{t("shifts.table.break_time")}</th>
            <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest border-b border-slate-100 text-right">{t("common.actions")}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-50">
          {shifts?.map(shift => (
            <tr key={shift.id} className="hover:bg-slate-50/50 transition-colors group">
              <td className="px-8 py-6">
                <div className="font-bold text-slate-900 text-lg">{shift.name}</div>
                <div className="text-[10px] text-slate-400 font-black tracking-widest uppercase mt-1">{shift.id}</div>
              </td>
              <td className="px-8 py-6">
                <div className="flex items-center gap-3">
                  <div className="bg-primary-50 text-blue-700 px-4 py-2 rounded-xl font-mono font-black text-sm border border-blue-100">
                    {formatTime(shift.start_time)}
                  </div>
                  <ChevronRight className="w-4 h-4 text-slate-200" />
                  <div className="bg-primary-50 text-blue-700 px-4 py-2 rounded-xl font-mono font-black text-sm border border-blue-100">
                    {formatTime(shift.end_time)}
                  </div>
                </div>
              </td>
              <td className="px-8 py-6">
                {shift.break_start && shift.break_end ? (
                  <div>
                    <div className="flex items-center gap-2 font-bold text-sm text-slate-700">
                      <span className="text-xs bg-slate-100 px-2 py-0.5 rounded text-slate-500 font-mono">{shift.break_start.substring(0, 5)}</span>
                      <span className="text-slate-300">-</span>
                      <span className="text-xs bg-slate-100 px-2 py-0.5 rounded text-slate-500 font-mono">{shift.break_end.substring(0, 5)}</span>
                    </div>
                    {shift.is_break_auto_deduct && (
                      <div className="text-[9px] font-black text-blue-500 uppercase tracking-tight mt-1 flex items-center gap-1">
                        <Zap size={8} /> {t("shifts.auto_deduct")}
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="flex items-center gap-2 font-bold text-sm text-slate-600">
                    <Clock className="w-4 h-4 text-slate-300" />
                    {shift.break_duration_minutes} {t("common.minutes")}
                  </div>
                )}
              </td>
              <td className="px-8 py-6 text-right">
                <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button 
                    onClick={() => onEdit(shift)}
                    className="p-3 bg-slate-50 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-2xl transition-all border border-transparent hover:border-blue-100"
                  >
                    <Save className="w-5 h-5" />
                  </button>
                  {onDelete && (
                    <button 
                        onClick={() => onDelete(shift)}
                        className="p-3 bg-slate-50 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-2xl transition-all border border-transparent hover:border-rose-100">
                      <Trash2 className="w-5 h-5" />
                    </button>
                  )}
                </div>
              </td>
            </tr>
          ))}
          {shifts?.length === 0 && (
            <tr>
              <td colSpan={4} className="py-32 text-center">
                <div className="bg-slate-50 w-20 h-20 rounded-[32px] flex items-center justify-center mx-auto mb-6">
                  <Clock size={32} className="text-slate-200" />
                </div>
                <h3 className="text-lg font-bold text-slate-900">{t("shifts.messages.no_shifts")}</h3>
                <p className="text-slate-500 max-w-[280px] mx-auto mt-2">{t("shifts.directory_title")} empty</p>
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
