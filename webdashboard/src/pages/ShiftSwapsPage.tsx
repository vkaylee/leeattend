import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
    ArrowLeftRight,
    Plus,
    Clock,
    CheckCircle,
    XCircle,
    Check,
    X,
    ArrowRight,
    CalendarDays,
    HelpCircle,
    ChevronDown,
    ChevronRight,
    Info,
} from "lucide-react";
import {
    fetchEmployees,
    getCurrentWorkspaceId,
    fetchShifts,
} from "../lib/api";
import {
    fetchShiftSwaps,
    createShiftSwap,
    approveShiftSwap,
    ShiftSwapStatus,
} from "../lib/api/shift-swaps";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useToast } from "@/components/ui/Toast";
import { cn } from "@/lib/utils";

type TabType = "pending_admin" | "approved" | "rejected";

export default function ShiftSwapsPage() {
    const { t } = useTranslation();
    const wsId = getCurrentWorkspaceId();
    const queryClient = useQueryClient();
    const { formatDate } = useDateTimeFormatter();
    const [activeTab, setActiveTab] = useState<TabType>("pending_admin");
    const [showCreateForm, setShowCreateForm] = useState(false);
    const [showGuide, setShowGuide] = useState(false);
    const toast = useToast();

    // Form state
    const [requesterId, setRequesterId] = useState("");
    const [targetId, setTargetId] = useState("");
    const [swapDate, setSwapDate] = useState("");
    const [reason, setReason] = useState("");

    const { data: swaps = [], isLoading } = useQuery({
        queryKey: ["shift-swaps", wsId, activeTab],
        queryFn: () => fetchShiftSwaps(wsId!, activeTab as ShiftSwapStatus),
        enabled: !!wsId,
    });

    const { data: employeesData } = useQuery({
        queryKey: ["employees", wsId],
        queryFn: () => fetchEmployees(wsId!, 1, 3000),
        enabled: !!wsId,
    });
    const employees = employeesData?.data || [];

    const createMutation = useMutation({
        mutationFn: (data: {
            requester_id: string;
            target_id: string;
            swap_date: string;
            reason?: string;
        }) => createShiftSwap(wsId!, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["shift-swaps"] });
            toast.success(t("shift_swaps.messages.create_success"));
            setShowCreateForm(false);
            setRequesterId("");
            setTargetId("");
            setSwapDate("");
            setReason("");
        },
        onError: () => {
            toast.error(t("shift_swaps.messages.create_error"));
        },
    });

    const approveMutation = useMutation({
        mutationFn: (data: { id: string; status: ShiftSwapStatus }) =>
            approveShiftSwap(wsId!, data.id, { status: data.status }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["shift-swaps"] });
            queryClient.invalidateQueries({ queryKey: ["employee-rosters"] });
            queryClient.invalidateQueries({ queryKey: ["workspace-schedules"] });
            toast.success(t("common.success"));
        },
    });

    const handleCreate = () => {
        if (!requesterId || !targetId || !swapDate) return;
        createMutation.mutate({
            requester_id: requesterId,
            target_id: targetId,
            swap_date: swapDate,
            reason: reason || undefined,
        });
    };

    if (!wsId)
        return (
            <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
                {t("common.select_workspace")}
            </div>
        );

    return (
        <div className="space-y-10 animate-in fade-in duration-700 pb-20">
            {/* Header */}
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
                <div className="space-y-1">
                    <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">
                        {t("shift_swaps.title")}
                    </h1>
                    <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">
                        {t("shift_swaps.subtitle")}
                    </p>
                </div>
                <button
                    onClick={() => setShowCreateForm(!showCreateForm)}
                    className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl hover:bg-primary-700 transition-all font-bold text-xs uppercase tracking-widest shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95"
                >
                    <Plus size={18} /> {t("shift_swaps.create_btn")}
                </button>
            </div>

            {/* Usage Guide */}
            <div className="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-950/30 dark:to-indigo-950/30 border border-blue-100 dark:border-blue-800/50 rounded-[2rem] overflow-hidden shadow-sm">
                <button
                    onClick={() => setShowGuide(!showGuide)}
                    className="flex items-center justify-between w-full px-8 py-5 group"
                >
                    <div className="flex items-center gap-3">
                        <div className="p-2 bg-blue-100 dark:bg-blue-900/50 rounded-xl">
                            <HelpCircle size={18} className="text-blue-600 dark:text-blue-400" />
                        </div>
                        <span className="text-sm font-bold text-blue-700 dark:text-blue-300 uppercase tracking-widest">
                            {t("shift_swaps.guide.title")}
                        </span>
                    </div>
                    {showGuide ? (
                        <ChevronDown size={16} className="text-blue-400" />
                    ) : (
                        <ChevronRight size={16} className="text-blue-400" />
                    )}
                </button>
                {showGuide && (
                    <div className="px-8 pb-6 space-y-4 animate-in slide-in-from-top-2 duration-300">
                        <p className="text-xs text-blue-700 dark:text-blue-300 leading-relaxed font-medium">
                            {t("shift_swaps.guide.intro")}
                        </p>

                        {/* Steps */}
                        <div className="space-y-3">
                            <div className="flex items-start gap-3">
                                <span className="flex-shrink-0 w-6 h-6 bg-blue-600 text-white rounded-full flex items-center justify-center text-[10px] font-black">1</span>
                                <div>
                                    <p className="text-xs font-bold text-slate-700 dark:text-slate-200">{t("shift_swaps.guide.step1_title")}</p>
                                    <p className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">{t("shift_swaps.guide.step1_desc")}</p>
                                </div>
                            </div>
                            <div className="flex items-start gap-3">
                                <span className="flex-shrink-0 w-6 h-6 bg-blue-600 text-white rounded-full flex items-center justify-center text-[10px] font-black">2</span>
                                <div>
                                    <p className="text-xs font-bold text-slate-700 dark:text-slate-200">{t("shift_swaps.guide.step2_title")}</p>
                                    <p className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">{t("shift_swaps.guide.step2_desc")}</p>
                                </div>
                            </div>
                            <div className="flex items-start gap-3">
                                <span className="flex-shrink-0 w-6 h-6 bg-blue-600 text-white rounded-full flex items-center justify-center text-[10px] font-black">3</span>
                                <div>
                                    <p className="text-xs font-bold text-slate-700 dark:text-slate-200">{t("shift_swaps.guide.step3_title")}</p>
                                    <p className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">{t("shift_swaps.guide.step3_desc")}</p>
                                </div>
                            </div>
                        </div>

                        {/* Important notes */}
                        <div className="mt-4 bg-amber-50 dark:bg-amber-950/30 border border-amber-100 dark:border-amber-800/50 rounded-xl p-4">
                            <div className="flex items-start gap-2">
                                <Info size={14} className="text-amber-600 dark:text-amber-400 mt-0.5 shrink-0" />
                                <div className="space-y-1">
                                    <p className="text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("shift_swaps.guide.notes_title")}</p>
                                    <ul className="text-[11px] text-amber-700 dark:text-amber-300/80 space-y-1 list-disc list-inside">
                                        <li>{t("shift_swaps.guide.note1")}</li>
                                        <li>{t("shift_swaps.guide.note2")}</li>
                                        <li>{t("shift_swaps.guide.note3")}</li>
                                    </ul>
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </div>

            {/* Create Form */}
            {showCreateForm && (
                <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-8 shadow-sm animate-in slide-in-from-top-4 duration-500">
                    <h3 className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest mb-6">
                        {t("shift_swaps.create_btn")}
                    </h3>
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                        <div>
                            <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2">
                                {t("shift_swaps.requester")}
                            </label>
                            <select
                                value={requesterId}
                                onChange={(e) => setRequesterId(e.target.value)}
                                className="w-full px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl text-sm text-slate-700 dark:text-slate-200 font-medium"
                            >
                                <option value="">{t("shift_swaps.select_employee")}</option>
                                {employees.map((emp) => (
                                    <option key={emp.id} value={emp.id}>
                                        {emp.full_name} ({emp.employee_id_number})
                                    </option>
                                ))}
                            </select>
                        </div>
                        <div>
                            <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2">
                                {t("shift_swaps.target")}
                            </label>
                            <select
                                value={targetId}
                                onChange={(e) => setTargetId(e.target.value)}
                                className="w-full px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl text-sm text-slate-700 dark:text-slate-200 font-medium"
                            >
                                <option value="">{t("shift_swaps.select_employee")}</option>
                                {employees
                                    .filter((emp) => emp.id !== requesterId)
                                    .map((emp) => (
                                        <option key={emp.id} value={emp.id}>
                                            {emp.full_name} ({emp.employee_id_number})
                                        </option>
                                    ))}
                            </select>
                        </div>
                        <div>
                            <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2">
                                {t("shift_swaps.swap_date")}
                            </label>
                            <input
                                type="date"
                                value={swapDate}
                                onChange={(e) => setSwapDate(e.target.value)}
                                className="w-full px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl text-sm text-slate-700 dark:text-slate-200 font-medium"
                            />
                        </div>
                        <div>
                            <label className="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2">
                                {t("shift_swaps.reason")}
                            </label>
                            <input
                                type="text"
                                value={reason}
                                onChange={(e) => setReason(e.target.value)}
                                placeholder={t("shift_swaps.reason_placeholder")}
                                className="w-full px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl text-sm text-slate-700 dark:text-slate-200 font-medium"
                            />
                        </div>
                    </div>
                    <div className="flex justify-end gap-3 mt-6">
                        <button
                            onClick={() => setShowCreateForm(false)}
                            className="px-6 py-2.5 rounded-xl text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-all"
                        >
                            {t("common.cancel")}
                        </button>
                        <button
                            onClick={handleCreate}
                            disabled={
                                !requesterId ||
                                !targetId ||
                                !swapDate ||
                                createMutation.isPending
                            }
                            className="px-6 py-2.5 rounded-xl text-xs font-bold uppercase tracking-widest bg-primary-600 text-white hover:bg-primary-700 shadow-lg disabled:opacity-50 transition-all active:scale-95"
                        >
                            {createMutation.isPending
                                ? t("common.loading")
                                : t("common.submit")}
                        </button>
                    </div>
                </div>
            )}

            {/* Status Tabs */}
            <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
                <button
                    onClick={() => setActiveTab("pending_admin")}
                    className={cn(
                        "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
                        activeTab === "pending_admin"
                            ? "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 shadow-sm border border-slate-100 dark:border-slate-800"
                            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200",
                    )}
                >
                    <Clock size={14} /> {t("shift_swaps.status.pending_admin")}
                </button>
                <button
                    onClick={() => setActiveTab("approved")}
                    className={cn(
                        "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
                        activeTab === "approved"
                            ? "bg-white dark:bg-slate-900 text-emerald-600 dark:text-emerald-300 shadow-sm border border-slate-100 dark:border-slate-800"
                            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200",
                    )}
                >
                    <CheckCircle size={14} /> {t("shift_swaps.status.approved")}
                </button>
                <button
                    onClick={() => setActiveTab("rejected")}
                    className={cn(
                        "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
                        activeTab === "rejected"
                            ? "bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 shadow-sm border border-slate-100 dark:border-slate-800"
                            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200",
                    )}
                >
                    <XCircle size={14} /> {t("shift_swaps.status.rejected")}
                </button>
            </div>

            {/* Table */}
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
                                <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                                    {t("shift_swaps.swap_date")}
                                </th>
                                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                                    {t("shift_swaps.requester")}
                                </th>
                                <th className="px-4 py-6 text-center">
                                    <ArrowLeftRight
                                        size={14}
                                        className="text-slate-400 mx-auto"
                                    />
                                </th>
                                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                                    {t("shift_swaps.target")}
                                </th>
                                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                                    {t("shift_swaps.reason")}
                                </th>
                                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
                                    {t("common.status")}
                                </th>
                                <th className="px-10 py-6"></th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                            {isLoading ? (
                                <tr>
                                    <td
                                        colSpan={7}
                                        className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse"
                                    >
                                        {t("common.loading")}
                                    </td>
                                </tr>
                            ) : swaps.length === 0 ? (
                                <tr>
                                    <td
                                        colSpan={7}
                                        className="px-10 py-32 text-center"
                                    >
                                        <div className="flex flex-col items-center gap-4">
                                            <ArrowLeftRight
                                                size={40}
                                                className="text-slate-200 dark:text-slate-700"
                                            />
                                            <span className="text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
                                                {t("shift_swaps.messages.no_data")}
                                            </span>
                                        </div>
                                    </td>
                                </tr>
                            ) : (
                                swaps.map((swap) => (
                                    <tr
                                        key={swap.id}
                                        className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group"
                                    >
                                        {/* Date */}
                                        <td className="px-10 py-6">
                                            <div className="inline-flex items-center gap-2 bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-inner">
                                                <CalendarDays
                                                    size={14}
                                                    className="text-primary-600 dark:text-primary-400"
                                                />
                                                <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tighter">
                                                    {formatDate(swap.swap_date)}
                                                </span>
                                            </div>
                                        </td>

                                        {/* Requester */}
                                        <td className="px-8 py-6">
                                            <div>
                                                <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1">
                                                    {swap.requester_name || "—"}
                                                </div>
                                                <div className="text-[10px] text-primary-600 dark:text-primary-400 font-bold">
                                                    {swap.requester_shift_name || t("shift_swaps.day_off")}
                                                </div>
                                            </div>
                                        </td>

                                        {/* Arrow */}
                                        <td className="px-4 py-6 text-center">
                                            <ArrowRight
                                                size={16}
                                                className="text-slate-300 dark:text-slate-600 mx-auto"
                                            />
                                        </td>

                                        {/* Target */}
                                        <td className="px-8 py-6">
                                            <div>
                                                <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1">
                                                    {swap.target_name || "—"}
                                                </div>
                                                <div className="text-[10px] text-primary-600 dark:text-primary-400 font-bold">
                                                    {swap.target_shift_name || t("shift_swaps.day_off")}
                                                </div>
                                            </div>
                                        </td>

                                        {/* Reason */}
                                        <td className="px-8 py-6">
                                            <p className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tight line-clamp-2 italic opacity-70 leading-relaxed">
                                                {swap.reason || t("common.no_notes")}
                                            </p>
                                        </td>

                                        {/* Status */}
                                        <td className="px-8 py-6 text-center">
                                            <span
                                                className={cn(
                                                    "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
                                                    swap.status === "pending_admin"
                                                        ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800"
                                                        : swap.status === "approved"
                                                            ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                                                            : "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800",
                                                )}
                                            >
                                                {t(`shift_swaps.status.${swap.status}`)}
                                            </span>
                                        </td>

                                        {/* Actions */}
                                        <td className="px-10 py-6 text-right">
                                            {swap.status === "pending_admin" && (
                                                <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">
                                                    <button
                                                        onClick={() =>
                                                            approveMutation.mutate({
                                                                id: swap.id,
                                                                status: "approved",
                                                            })
                                                        }
                                                        className="p-3 bg-emerald-600 text-white hover:bg-emerald-700 rounded-xl shadow-lg shadow-emerald-100 transition-all active:scale-90"
                                                        title={t("common.approve")}
                                                    >
                                                        <Check size={18} />
                                                    </button>
                                                    <button
                                                        onClick={() =>
                                                            approveMutation.mutate({
                                                                id: swap.id,
                                                                status: "rejected",
                                                            })
                                                        }
                                                        className="p-3 bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 rounded-xl shadow-md border border-rose-100 dark:border-rose-800 transition-all active:scale-90"
                                                        title={t("common.reject")}
                                                    >
                                                        <X size={18} />
                                                    </button>
                                                </div>
                                            )}
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}
