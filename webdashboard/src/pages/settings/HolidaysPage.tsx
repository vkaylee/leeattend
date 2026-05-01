import { useState, useEffect, useCallback } from "react";

import { Plus, Trash2, Pencil, CalendarDays, ChevronLeft, ChevronRight, Info } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/Combobox";
import { useConfirm } from "@/hooks/use-confirm";

import { getCurrentWorkspaceId } from "@/lib/api";
import { fetchHolidays, createHoliday, updateHoliday, deleteHoliday } from "@/lib/api/holidays";
import { fetchLeaveTypes } from "@/lib/api/leaves";

import type { Holiday, LeaveType } from "@/lib/api/types";

export default function HolidaysPage() {
    const { t } = useTranslation();
    const [holidays, setHolidays] = useState<Holiday[]>([]);
    const [leaveTypes, setLeaveTypes] = useState<LeaveType[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [year, setYear] = useState(new Date().getFullYear());
    const [showForm, setShowForm] = useState(false);
    const [editingHoliday, setEditingHoliday] = useState<Holiday | null>(null);
    const [error, setError] = useState<string | null>(null);
    const { confirm, ConfirmationDialog } = useConfirm();

    // Form state
    const [formName, setFormName] = useState("");
    const [formDate, setFormDate] = useState("");
    const [formType, setFormType] = useState("national");
    const [formIsPaid, setFormIsPaid] = useState(true);
    const [formLeaveTypeId, setFormLeaveTypeId] = useState("");
    const [formOtMultiplier, setFormOtMultiplier] = useState("");
    const [formDescription, setFormDescription] = useState("");
    const [formIsCreditOnWork, setFormIsCreditOnWork] = useState(true);
    const [formThresholdPercent, setFormThresholdPercent] = useState("");

    const wsId = getCurrentWorkspaceId();

    const loadHolidays = useCallback(async () => {
        if (!wsId) return;
        setIsLoading(true);
        try {
            const data = await fetchHolidays(wsId, year);
            setHolidays(data);
        } catch {
            setError(t('holidays.messages.load_error'));
        } finally {
            setIsLoading(false);
        }
    }, [wsId, year, t]);

    useEffect(() => {
        loadHolidays();
    }, [loadHolidays]);

    useEffect(() => {
        if (!wsId) return;
        fetchLeaveTypes(wsId).then(setLeaveTypes).catch(() => { });
    }, [wsId]);

    const resetForm = () => {
        setFormName("");
        setFormDate("");
        setFormType("national");
        setFormIsPaid(true);
        setFormLeaveTypeId("");
        setFormOtMultiplier("");
        setFormDescription("");
        setFormIsCreditOnWork(true);
        setFormThresholdPercent("");
        setEditingHoliday(null);
        setShowForm(false);
        setError(null);
    };

    const openEdit = (h: Holiday) => {
        setEditingHoliday(h);
        setFormName(h.name);
        setFormDate(h.date);
        setFormType(h.holiday_type);
        setFormIsPaid(h.is_paid);
        setFormLeaveTypeId(h.leave_type_id || "");
        setFormOtMultiplier(h.ot_multiplier?.toString() || "");
        setFormDescription(h.description || "");
        setFormIsCreditOnWork(h.is_credit_on_work ?? true);
        setFormThresholdPercent(h.work_threshold_percent?.toString() || "");
        setShowForm(true);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const currentWsId = wsId;
        if (!currentWsId) return;
        setError(null);
 

        const payload = {
            name: formName,
            date: formDate,
            holiday_type: formType,
            is_paid: formIsPaid,
            leave_type_id: !formIsPaid && formLeaveTypeId ? formLeaveTypeId : null,
            description: formDescription || undefined,
            ot_multiplier: formOtMultiplier ? parseFloat(formOtMultiplier) : undefined,
            is_credit_on_work: formIsCreditOnWork,
            work_threshold_percent: formThresholdPercent ? parseInt(formThresholdPercent) : null,
        } as Parameters<typeof createHoliday>[1];

        try {
            if (editingHoliday) {
                await updateHoliday(currentWsId, editingHoliday.id, payload);
            } else {
                await createHoliday(currentWsId, payload);
            }
            resetForm();
 
            loadHolidays();
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : t('holidays.messages.save_error'));
        }
    };

    const handleDelete = async (id: string) => {
        if (!wsId) return;
        const ok = await confirm({
            title: t('common.confirm_delete', 'Confirm Delete'),
            message: t('holidays.messages.confirm_delete'),
            variant: 'danger'
        });
        if (!ok) return;
        try {
            await deleteHoliday(wsId, id);
            loadHolidays();
        } catch {
            setError(t('holidays.messages.delete_error'));
        }
    };

    return (
        <div className="space-y-8">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 dark:text-white tracking-tight">
                        🏖️ {t('holidays.title')}
                    </h1>
                    <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                        {t('holidays.subtitle')}
                    </p>
                </div>
                <button
                    onClick={() => { resetForm(); setShowForm(true); }}
                    className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-xl font-bold text-sm hover:bg-primary-700 transition-colors shadow-sm"
                >
                    <Plus size={16} />
                    {t('holidays.add_btn')}
                </button>
            </div>

            {/* Year Navigation */}
            <div className="flex items-center justify-center gap-4">
                <button
                    onClick={() => setYear(y => y - 1)}
                    className="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
                >
                    <ChevronLeft size={16} />
                </button>
                <span className="text-xl font-bold text-slate-900 dark:text-white tabular-nums">
                    {year}
                </span>
                <button
                    onClick={() => setYear(y => y + 1)}
                    className="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
                >
                    <ChevronRight size={16} />
                </button>
            </div>

            {/* Error */}
            {error && (
                <div className="p-3 bg-rose-50 dark:bg-rose-950 border border-rose-200 dark:border-rose-800 rounded-xl text-rose-600 dark:text-rose-300 text-sm font-medium">
                    {error}
                </div>
            )}

            {/* Form Modal */}
            {showForm && (
                <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-800 p-6 shadow-lg">
                    <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-4">
                        {editingHoliday ? t('holidays.modal_title_edit') : t('holidays.modal_title_new')}
                    </h2>
                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                    {t('holidays.fields.name')} *
                                </label>
                                <input
                                    type="text"
                                    value={formName}
                                    onChange={(e) => setFormName(e.target.value)}
                                    required
                                    placeholder={t('holidays.fields.name_placeholder')}
                                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                    {t('holidays.fields.date')} *
                                </label>
                                <input
                                    type="date"
                                    value={formDate}
                                    onChange={(e) => setFormDate(e.target.value)}
                                    required
                                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                    {t('holidays.fields.type')}
                                </label>
                                <Combobox
                                    value={formType}
                                    onChange={setFormType}
                                    options={[
                                        { value: "national", label: t('holidays.fields.types.national') },
                                        { value: "company", label: t('holidays.fields.types.company') },
                                        { value: "regional", label: t('holidays.fields.types.regional') },
                                    ]}
                                    placeholder={t('holidays.fields.type')}
                                    className="w-full border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm py-[7px]"
                                />
                            </div>
                        </div>
                        <div>
                            <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                {t('holidays.fields.description')}
                            </label>
                            <input
                                type="text"
                                value={formDescription}
                                onChange={(e) => setFormDescription(e.target.value)}
                                placeholder={t('holidays.fields.description_placeholder')}
                                className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                            />
                        </div>
                        <div className="flex items-center gap-2">
                            <input
                                type="checkbox"
                                id="isPaid"
                                checked={formIsPaid}
                                onChange={(e) => { setFormIsPaid(e.target.checked); if (e.target.checked) setFormLeaveTypeId(""); }}
                                className="rounded"
                            />
                            <label htmlFor="isPaid" className="text-sm text-slate-700 dark:text-slate-300">
                                {t('holidays.fields.paid_holiday')}
                            </label>
                        </div>
                        {!formIsPaid && (
                            <div className="space-y-3">
                                <div className="p-3 bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800/50 rounded-xl flex items-start gap-3">
                                    <Info className="w-5 h-5 text-blue-500 dark:text-blue-400 mt-0.5 shrink-0" />
                                    <div>
                                        <h4 className="text-sm font-bold text-blue-900 dark:text-blue-200 mb-1">
                                            {t('holidays.info.title')}
                                        </h4>
                                        <p className="text-xs text-blue-800 dark:text-blue-300 font-medium mb-2">
                                            {t('holidays.info.body_1')}
                                        </p>
                                        <ul className="text-xs text-blue-800 dark:text-blue-300 space-y-1 list-disc list-inside">
                                            <li><span className="font-bold">{t('holidays.info.rule_1').split(':')[0]}:</span> {t('holidays.info.rule_1').split(':')[1]}</li>
                                            <li><span className="font-bold">{t('holidays.info.rule_2').split(':')[0]}:</span> {t('holidays.info.rule_2').split(':')[1]}</li>
                                        </ul>
                                    </div>
                                </div>
                                <div>
                                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                        {t('holidays.fields.deduct_leave_type')}
                                    </label>
                                    <Combobox
                                        value={formLeaveTypeId}
                                        onChange={setFormLeaveTypeId}
                                        options={leaveTypes.filter(lt => lt.is_paid).map(lt => ({
                                            value: lt.id,
                                            label: `${lt.name} (${lt.code})`
                                        }))}
                                        placeholder={t('holidays.fields.auto_first_paid')}
                                        className="w-full border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm py-[7px]"
                                    />
                                    <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
                                        ⚠️ {t('holidays.fields.deduction_warning')}
                                    </p>
                                </div>
                            </div>
                        )}
                        {/* Work Policy Section */}
                        <div className="mt-2 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-xl border border-slate-200 dark:border-slate-700">
                            <h3 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-3 flex items-center gap-1.5">
                                ⚙️ {t('holidays.fields.work_policy_title', 'Chính sách khi NV đi làm ngày lễ')}
                            </h3>
                            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                                <div className="flex items-start gap-2">
                                    <input
                                        type="checkbox"
                                        id="isCreditOnWork"
                                        checked={formIsCreditOnWork}
                                        onChange={(e) => setFormIsCreditOnWork(e.target.checked)}
                                        className="rounded mt-0.5"
                                    />
                                    <div>
                                        <label htmlFor="isCreditOnWork" className="text-sm text-slate-700 dark:text-slate-300 font-medium">
                                            {t('holidays.fields.credit_on_work', 'Credit lương ngày lễ')}
                                        </label>
                                        <p className="text-[10px] text-slate-400">
                                            {t('holidays.fields.credit_on_work_help', 'NV vẫn nhận lương ngày lễ + lương giờ làm')}
                                        </p>
                                    </div>
                                </div>
                                <div>
                                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                        {t('holidays.fields.threshold_percent', 'Ngưỡng hoàn phép')} (%)
                                    </label>
                                    <input
                                        type="number"
                                        min={0}
                                        max={100}
                                        value={formThresholdPercent}
                                        onChange={(e) => setFormThresholdPercent(e.target.value)}
                                        placeholder="50"
                                        className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                                    />
                                    <p className="text-[10px] text-slate-400 mt-0.5">
                                        {t('holidays.fields.threshold_help', 'NV làm ≥ ngưỡng → hoàn phép. < ngưỡng → giữ phép')}
                                    </p>
                                </div>
                                <div>
                                    <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                        {t('holidays.fields.ot_multiplier', 'Hệ số nhân giờ làm')}
                                    </label>
                                    <input
                                        type="number"
                                        step="0.1"
                                        value={formOtMultiplier}
                                        onChange={(e) => setFormOtMultiplier(e.target.value)}
                                        placeholder={t('holidays.fields.ot_multiplier_placeholder', '3.0')}
                                        className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                                    />
                                    <p className="text-[10px] text-slate-400 mt-0.5">
                                        {t('holidays.fields.ot_multiplier_help', 'VD: 3× = 300% lương cho mỗi giờ đi làm')}
                                    </p>
                                </div>
                            </div>
                        </div>
                        <div className="flex gap-2 justify-end">
                            <button
                                type="button"
                                onClick={resetForm}
                                className="px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-700 text-sm font-bold text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800"
                            >
                                {t('common.cancel')}
                            </button>
                            <button
                                type="submit"
                                className="px-4 py-2 bg-primary-600 text-white rounded-xl font-bold text-sm hover:bg-primary-700 transition-colors"
                            >
                                {editingHoliday ? t('common.update') : t('common.create')}
                            </button>
                        </div>
                    </form>
                </div>
            )}

            {/* Holidays Table */}
            <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                {isLoading ? (
                    <div className="p-12 text-center text-slate-400 text-sm font-bold uppercase tracking-widest animate-pulse">
                        {t('common.loading')}
                    </div>
                ) : holidays.length === 0 ? (
                    <div className="p-12 text-center">
                        <CalendarDays className="w-12 h-12 text-slate-300 dark:text-slate-600 mx-auto mb-3" />
                        <p className="text-slate-400 text-sm font-medium">
                            {t('holidays.no_holidays', { year })}
                        </p>
                    </div>
                ) : (
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
                                <tr key={h.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group">
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
                                                {h.leave_type_id && leaveTypes.find(lt => lt.id === h.leave_type_id) && (
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
                                        {h.description || "—"}
                                    </td>
                                    <td className="px-4 py-3">
                                        <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                            <button
                                                onClick={() => openEdit(h)}
                                                className="p-1.5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-primary-600 hover:bg-primary-50 transition-colors"
                                            >
                                                <Pencil size={14} />
                                            </button>
                                            <button
                                                onClick={() => handleDelete(h.id)}
                                                className="p-1.5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-rose-600 hover:bg-rose-50 transition-colors"
                                            >
                                                <Trash2 size={14} />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </div>
            <ConfirmationDialog />
        </div>
    );
}
