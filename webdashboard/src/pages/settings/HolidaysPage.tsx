import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Trash2, Pencil, CalendarDays, ChevronLeft, ChevronRight } from "lucide-react";
import { fetchHolidays, createHoliday, updateHoliday, deleteHoliday } from "@/lib/api/holidays";
import { getCurrentWorkspaceId } from "@/lib/api";
import type { Holiday } from "@/lib/api/types";

export default function HolidaysPage() {
    const { t } = useTranslation();
    const [holidays, setHolidays] = useState<Holiday[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [year, setYear] = useState(new Date().getFullYear());
    const [showForm, setShowForm] = useState(false);
    const [editingHoliday, setEditingHoliday] = useState<Holiday | null>(null);
    const [error, setError] = useState<string | null>(null);

    // Form state
    const [formName, setFormName] = useState("");
    const [formDate, setFormDate] = useState("");
    const [formType, setFormType] = useState("national");
    const [formIsPaid, setFormIsPaid] = useState(true);
    const [formOtMultiplier, setFormOtMultiplier] = useState("");
    const [formDescription, setFormDescription] = useState("");

    const wsId = getCurrentWorkspaceId();

    const loadHolidays = useCallback(async () => {
        if (!wsId) return;
        setIsLoading(true);
        try {
            const data = await fetchHolidays(wsId, year);
            setHolidays(data);
        } catch {
            setError("Failed to load holidays");
        } finally {
            setIsLoading(false);
        }
    }, [wsId, year]);

    useEffect(() => {
        loadHolidays();
    }, [loadHolidays]);

    const resetForm = () => {
        setFormName("");
        setFormDate("");
        setFormType("national");
        setFormIsPaid(true);
        setFormOtMultiplier("");
        setFormDescription("");
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
        setFormOtMultiplier(h.ot_multiplier?.toString() || "");
        setFormDescription(h.description || "");
        setShowForm(true);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!wsId) return;
        setError(null);

        const payload: any = {
            name: formName,
            date: formDate,
            holiday_type: formType,
            is_paid: formIsPaid,
            description: formDescription || undefined,
            ot_multiplier: formOtMultiplier ? parseFloat(formOtMultiplier) : undefined,
        };

        try {
            if (editingHoliday) {
                await updateHoliday(wsId, editingHoliday.id, payload);
            } else {
                await createHoliday(wsId, payload);
            }
            resetForm();
            loadHolidays();
        } catch (err: any) {
            setError(err.message || "Failed to save holiday");
        }
    };

    const handleDelete = async (id: string) => {
        if (!wsId) return;
        if (!confirm("Are you sure you want to delete this holiday?")) return;
        try {
            await deleteHoliday(wsId, id);
            loadHolidays();
        } catch {
            setError("Failed to delete holiday");
        }
    };

    return (
        <div className="space-y-8">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 dark:text-white tracking-tight">
                        🏖️ Holidays
                    </h1>
                    <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                        Manage public holidays and special days for your workspace
                    </p>
                </div>
                <button
                    onClick={() => { resetForm(); setShowForm(true); }}
                    className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-xl font-bold text-sm hover:bg-primary-700 transition-colors shadow-sm"
                >
                    <Plus size={16} />
                    Add Holiday
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
                        {editingHoliday ? "Edit Holiday" : "New Holiday"}
                    </h2>
                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                    Name *
                                </label>
                                <input
                                    type="text"
                                    value={formName}
                                    onChange={(e) => setFormName(e.target.value)}
                                    required
                                    placeholder="e.g. Tết Dương Lịch"
                                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                    Date *
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
                                    Type
                                </label>
                                <select
                                    value={formType}
                                    onChange={(e) => setFormType(e.target.value)}
                                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                                >
                                    <option value="national">National</option>
                                    <option value="company">Company</option>
                                    <option value="regional">Regional</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                    OT Multiplier
                                </label>
                                <input
                                    type="number"
                                    step="0.1"
                                    value={formOtMultiplier}
                                    onChange={(e) => setFormOtMultiplier(e.target.value)}
                                    placeholder="3.0"
                                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                                />
                            </div>
                        </div>
                        <div>
                            <label className="block text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                                Description
                            </label>
                            <input
                                type="text"
                                value={formDescription}
                                onChange={(e) => setFormDescription(e.target.value)}
                                placeholder="Optional description"
                                className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm"
                            />
                        </div>
                        <div className="flex items-center gap-2">
                            <input
                                type="checkbox"
                                id="isPaid"
                                checked={formIsPaid}
                                onChange={(e) => setFormIsPaid(e.target.checked)}
                                className="rounded"
                            />
                            <label htmlFor="isPaid" className="text-sm text-slate-700 dark:text-slate-300">
                                Paid holiday
                            </label>
                        </div>
                        <div className="flex gap-2 justify-end">
                            <button
                                type="button"
                                onClick={resetForm}
                                className="px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-700 text-sm font-bold text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800"
                            >
                                Cancel
                            </button>
                            <button
                                type="submit"
                                className="px-4 py-2 bg-primary-600 text-white rounded-xl font-bold text-sm hover:bg-primary-700 transition-colors"
                            >
                                {editingHoliday ? "Update" : "Create"}
                            </button>
                        </div>
                    </form>
                </div>
            )}

            {/* Holidays Table */}
            <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                {isLoading ? (
                    <div className="p-12 text-center text-slate-400 text-sm font-bold uppercase tracking-widest animate-pulse">
                        Loading...
                    </div>
                ) : holidays.length === 0 ? (
                    <div className="p-12 text-center">
                        <CalendarDays className="w-12 h-12 text-slate-300 dark:text-slate-600 mx-auto mb-3" />
                        <p className="text-slate-400 text-sm font-medium">
                            No holidays configured for {year}
                        </p>
                    </div>
                ) : (
                    <table className="w-full text-left">
                        <thead>
                            <tr className="border-b border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/50">
                                <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">Date</th>
                                <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">Name</th>
                                <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">Type</th>
                                <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">Paid</th>
                                <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-center">OT ×</th>
                                <th className="px-4 py-3 text-[10px] font-bold text-slate-500 uppercase tracking-widest">Description</th>
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
                                            {h.holiday_type}
                                        </span>
                                    </td>
                                    <td className="px-4 py-3 text-center">
                                        {h.is_paid ? "✅" : "❌"}
                                    </td>
                                    <td className="px-4 py-3 text-center font-mono font-bold text-sm text-orange-600 dark:text-orange-400">
                                        {h.ot_multiplier ? `${h.ot_multiplier}×` : "—"}
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
        </div>
    );
}
