import React from "react";
import { useTranslation } from "react-i18next";
import { History, Zap, ShieldCheck } from "lucide-react";
import { AssignmentHistory as AssignmentHistoryType, Employee } from "../../lib/api";
import { useDateTimeFormatter } from "../../hooks/use-datetime";

interface AssignmentHistoryProps {
  history: AssignmentHistoryType[] | undefined;
  selectedEmployee: Employee | null;
}

export function AssignmentHistory({ history, selectedEmployee }: AssignmentHistoryProps) {
  const { t } = useTranslation();
  const { formatDateTime } = useDateTimeFormatter();

  return (
    <div className="bg-white border border-slate-200 rounded-[40px] p-8 shadow-sm animate-in fade-in slide-in-from-bottom-8 duration-500">
      <div className="flex items-center gap-3 mb-6">
        <History className="w-5 h-5 text-slate-400" />
        <h3 className="text-sm font-black text-slate-900 uppercase tracking-widest">{t("shifts.history_title")}</h3>
      </div>
      
      <div className="overflow-hidden rounded-3xl border border-slate-100">
        <table className="w-full text-left">
          <thead className="bg-slate-50/50">
            <tr>
              <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.time")}</th>
              <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
              <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.table.assignment_type")}</th>
              <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.shift")}</th>
              <th className="p-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.table.range")}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {history?.filter(h => h.employee_id === selectedEmployee?.id || !selectedEmployee).map(h => (
              <tr key={h.id} className="hover:bg-slate-50/50">
                <td className="p-4 text-xs font-bold text-slate-500">
                  {formatDateTime(h.created_at)}
                </td>
                <td className="p-4 text-sm font-bold text-slate-900">{h.employee_name}</td>
                <td className="p-4">
                  {h.assignment_type === 'roster' ? (
                    <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-purple-50 text-purple-600 text-[10px] font-black uppercase tracking-tighter border border-purple-100">
                      <Zap size={10} /> {t("shifts.types.flexible")}
                    </span>
                  ) : (
                    <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-emerald-50 text-emerald-600 text-[10px] font-black uppercase tracking-tighter border border-emerald-100">
                      <ShieldCheck size={10} /> {t("shifts.types.fixed")}
                    </span>
                  )}
                </td>
                <td className="p-4 font-bold text-slate-700 text-xs">{h.shift_name || t("shifts.off_day")}</td>
                <td className="p-4 font-mono text-xs text-slate-500">{h.date_range}</td>
              </tr>
            ))}
            {(!history || history.length === 0) && (
              <tr>
                <td colSpan={5} className="p-8 text-center text-xs font-bold text-slate-300 uppercase tracking-widest">{t("common.no_history")}</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
