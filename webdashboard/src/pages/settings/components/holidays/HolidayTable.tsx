import { CalendarDays, Pencil, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Holiday, LeaveType } from "@/lib/api/types";

interface HolidayTableProps {
    isLoading: boolean;
    holidays: Holiday[];
    year: number;
    leaveTypes: LeaveType[];
    onEdit: (h: Holiday) => void;
    onDelete: (id: string) => Promise<void>;
}

export function HolidayTable({ isLoading, holidays, year, leaveTypes, onEdit, onDelete }: HolidayTableProps) {
    const { t } = useTranslation();

    if (isLoading) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                <div className="p-12 text-center text-slate-400 text-sm font-bold uppercase tracking-widest animate-pulse">
                    {t('common.loading')}
                </div>
            </div>
        );
    }

    if (holidays.length === 0) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                <div className="p-12 text-center">
                    <CalendarDays className="w-12 h-12 text-slate-300 dark:text-slate-600 mx-auto mb-3" />
                    <p className="text-slate-400 text-sm font-medium">
                        {t('holidays.no_holidays', { year })}
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
            <table className="w-full text-left">
                <thead>
                    <tr className="border-b border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/50">
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t('holidays.table.date')}</th>
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t('holidays.table.name')}</th>
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t('holidays.table.type')}</th>
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">{t('holidays.table.paid')}</th>
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">{t('holidays.table.ot')}</th>
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">{t('holidays.table.credit', 'Credit')}</th>
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">{t('holidays.table.threshold', 'Ngưỡng')}</th>
                        <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t('holidays.table.description')}</th>
                        <th className="px-4 py-3"></th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                    {holidays.map((h) => (
                        <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group" key={h.id}>
                            <td className="px-4 py-3 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">
                                {h.date}
                            </td>
                            <td className="px-4 py-3 font-bold text-slate-900 dark:text-white text-sm">
                                {h.name}
                            </td>
                            <td className="px-4 py-3">
                                <span className="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wide bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 border border-slate-200 dark:border-slate-700">
                                    {t(`holidays.fields.types.${h.holiday_type}`)}
                                </span>
                            </td>
                            <td className="px-4 py-3 text-center">
                                {h.is_paid ? (
                                    <span className="text-emerald-600">✅ {t('holidays.table.paid_status')}</span>
                                ) : (
                                    <span className="text-amber-600 text-xs font-bold">
                                        ⚡ {t('holidays.table.deduct_leave')}
                                        {!!h.leave_type_id && leaveTypes.find(lt => lt.id === h.leave_type_id) && (
                                            <span className="block text-[10px] text-slate-400 font-normal">
                                                → {leaveTypes.find(lt => lt.id === h.leave_type_id)?.name}
                                            </span>
                                          )}
                                    </span>
                                )}
                            </td>
                            <td className="px-4 py-3 text-center font-mono font-bold text-sm text-orange-600 dark:text-orange-400">
                                {h.ot_multiplier ? `${h.ot_multiplier}×` : "—"}
                            </td>
                            <td className="px-4 py-3 text-center text-sm">
                                {h.is_credit_on_work ? (
                                    <span className="text-emerald-600">✅</span>
                                ) : (
                                    <span className="text-slate-400">❌</span>
                                )}
                            </td>
                            <td className="px-4 py-3 text-center font-mono text-sm text-slate-600 dark:text-slate-400">
                                {h.work_threshold_percent != null ? `${h.work_threshold_percent}%` : "—"}
                            </td>
                            <td className="px-4 py-3 text-xs text-slate-500 dark:text-slate-400 max-w-[200px] truncate">
                                {h.description ?? "—"}
                            </td>
                            <td className="px-4 py-3">
                                <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <button
                                        className="p-1.5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-primary-600 hover:bg-primary-50 transition-colors"
                                        onClick={() => { onEdit(h); }}
                                    >
                                        <Pencil size={14} />
                                    </button>
                                    <button
                                        className="p-1.5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-rose-600 hover:bg-rose-50 transition-colors"
                                        onClick={() => onDelete(h.id)}
                                    >
                                        <Trash2 size={14} />
                                    </button>
                                </div>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
