import React from "react";

import { ChevronRight, Clock, Zap, Pencil, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useDateTimeFormatter } from "../../hooks/use-datetime";
import { Shift } from "../../lib/api";

interface ShiftListProps {
  shifts: Shift[] | undefined;
  onEdit: (shift: Shift) => void;
  onDelete?: (shift: Shift) => void;
}

export function ShiftList({ shifts, onEdit, onDelete }: ShiftListProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="p-8 border-b border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex items-center justify-between">
        <h2 className="font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("shifts.directory_title")}</h2>
        <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm">{t("common.total").toUpperCase()}: {shifts?.length || 0}</span>
      </div>
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/10">
            <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("shifts.table.shift_name")}</th>
            <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800 text-center">{t("shifts.table.time_range")}</th>
            <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800 text-center">{t("shifts.table.break_time")}</th>
            <th className="px-8 py-5 border-b border-slate-50 dark:border-slate-800"></th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {shifts?.map(shift => (
            <tr key={shift.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
              <td className="px-8 py-6">
                <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{shift.name}</div>
                <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold tracking-tighter mt-1">{shift.id}</div>
              </td>
              <td className="px-8 py-6 text-center">
                <div className="flex items-center justify-center gap-3">
                  <div className="bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 px-4 py-2 rounded-xl font-mono font-bold text-xs border border-primary-100 dark:border-primary-800 shadow-sm">
                    {formatTime(shift.start_time)}
                  </div>
                  <ChevronRight className="w-4 h-4 text-slate-300 dark:text-slate-600" />
                  <div className="bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 px-4 py-2 rounded-xl font-mono font-bold text-xs border border-primary-100 dark:border-primary-800 shadow-sm">
                    {formatTime(shift.end_time)}
                  </div>
                </div>
              </td>
              <td className="px-8 py-6 text-center">
                {shift.break_start && shift.break_end ? (
                  <div className="flex flex-col items-center gap-1.5">
                    <div className="flex items-center gap-2 px-3 py-1 bg-slate-50 dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-800 shadow-inner">
                      <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">{shift.break_start.substring(0, 5)}</span>
                      <span className="text-slate-300 dark:text-slate-600 font-bold">-</span>
                      <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">{shift.break_end.substring(0, 5)}</span>
                    </div>
                    {shift.is_break_auto_deduct && (
                      <div className="text-[8px] font-bold text-blue-500 uppercase tracking-widest flex items-center gap-1 opacity-80">
                        <Zap size={10} className="fill-blue-500" /> {t("shifts.auto_deduct")}
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="flex items-center justify-center gap-2 font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    <Clock size={14} className="text-slate-200 dark:text-slate-700" />
                    {shift.break_duration_minutes} {t("common.minutes")}
                  </div>
                )}
              </td>
              <td className="px-8 py-6 text-right">
                <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-all duration-300">
                  <button onClick={() => onEdit(shift)} className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.edit")}><Pencil size={18} /></button>
                  {onDelete && (
                    <button onClick={() => onDelete(shift)} className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.delete")}><Trash2 size={18} /></button>
                  )}
                </div>
              </td>
            </tr>
          ))}
          {shifts?.length === 0 && (
            <tr>
              <td colSpan={4} className="py-32 text-center bg-slate-50 dark:bg-slate-800/30">
                <div className="bg-white dark:bg-slate-900 w-20 h-20 rounded-[2rem] flex items-center justify-center mx-auto mb-6 shadow-sm border border-slate-100 dark:border-slate-800">
                  <Clock size={32} className="text-slate-200 dark:text-slate-700" />
                </div>
                <h3 className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">{t("shifts.messages.no_shifts")}</h3>
                <p className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-1 opacity-50">{t("shifts.directory_title")} is empty</p>
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
