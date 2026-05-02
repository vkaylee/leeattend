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
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import ApprovalListLayout, { TabConfig } from "@/components/templates/ApprovalListLayout";
import Button from "@/components/ui/Button";
import { Combobox } from "@/components/ui/Combobox";
import FormField from "@/components/ui/FormField";
import StatusBadge from "@/components/ui/StatusBadge";
import { useToast } from "@/components/ui/Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import { cn } from "@/lib/utils";

import {
    getCurrentWorkspaceId,
    
} from "../lib/api";
import {
    fetchShiftSwaps,
    createShiftSwap,
    approveShiftSwap,
    fetchShiftSwapStats,
    ShiftSwapStatus,
} from "../lib/api/shift-swaps";

type TabType = "pending_target" | "pending_admin" | "approved" | "rejected" | "cancelled";

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
        queryFn: () => fetchShiftSwaps(wsId!, activeTab),
        enabled: !!wsId,
    });

    const { data: stats } = useQuery({
        queryKey: ["shift-swap-stats", wsId],
        queryFn: () => fetchShiftSwapStats(wsId!),
        enabled: !!wsId,
    });

    const { employees } = useAllEmployees(wsId);

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

    const tabs: TabConfig[] = [
        { id: "pending_admin", label: t("shift_swaps.status.pending_admin"), icon: <Clock size={14} />, colorScheme: "amber" },
        { id: "approved", label: t("shift_swaps.status.approved"), icon: <CheckCircle size={14} />, colorScheme: "emerald" },
        { id: "rejected", label: t("shift_swaps.status.rejected"), icon: <XCircle size={14} />, colorScheme: "rose" },
        { id: "cancelled", label: t("shift_swaps.status.cancelled"), icon: <XCircle size={14} />, colorScheme: "slate" },
    ];

    const tableHeaders = [
        <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" key="date">
            {t("shift_swaps.swap_date")}
        </th>,
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" key="requester">
            {t("shift_swaps.requester")}
        </th>,
        <th className="px-4 py-6 text-center" key="arrow">
            <ArrowLeftRight className="text-slate-400 mx-auto" size={14} />
        </th>,
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" key="target">
            {t("shift_swaps.target")}
        </th>,
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]" key="reason">
            {t("shift_swaps.reason")}
        </th>,
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center" key="status">
            {t("common.status")}
        </th>,
        <th className="px-10 py-6" key="actions"></th>,
    ];

    return (
        <ApprovalListLayout
            activeTab={activeTab}
            banner={
                <>
                    {/* Usage Guide */}
                    <div className="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-950/30 dark:to-indigo-950/30 border border-blue-100 dark:border-blue-800/50 rounded-[2rem] overflow-hidden shadow-sm">
                        <button
                            className="flex items-center justify-between w-full px-8 py-5 group"
                            onClick={() => { setShowGuide(!showGuide); }}
                        >
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-blue-100 dark:bg-blue-900/50 rounded-xl">
                                    <HelpCircle className="text-blue-600 dark:text-blue-400" size={18} />
                                </div>
                                <span className="text-sm font-bold text-blue-700 dark:text-blue-300 uppercase tracking-widest">
                                    {t("shift_swaps.guide.title")}
                                </span>
                            </div>
                            {showGuide ? (
                                <ChevronDown className="text-blue-400" size={16} />
                            ) : (
                                <ChevronRight className="text-blue-400" size={16} />
                            )}
                        </button>

                        {!!showGuide && (
                            <div className="px-8 pb-8 animate-in slide-in-from-top-4 duration-300">
                                <div className="grid grid-cols-1 md:grid-cols-3 gap-6 pt-4 border-t border-blue-100 dark:border-blue-800/30">
                                    <div className="space-y-3">
                                        <div className="w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400 flex items-center justify-center font-bold font-mono text-xs">
                                            1
                                        </div>
                                        <h4 className="font-bold text-sm text-slate-700 dark:text-slate-300">
                                            {t("shift_swaps.guide.step1.title")}
                                        </h4>
                                        <p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider leading-relaxed opacity-80">
                                            {t("shift_swaps.guide.step1.desc")}
                                        </p>
                                    </div>
                                    <div className="space-y-3">
                                        <div className="w-8 h-8 rounded-full bg-indigo-100 dark:bg-indigo-900/50 text-indigo-600 dark:text-indigo-400 flex items-center justify-center font-bold font-mono text-xs">
                                            2
                                        </div>
                                        <h4 className="font-bold text-sm text-slate-700 dark:text-slate-300">
                                            {t("shift_swaps.guide.step2.title")}
                                        </h4>
                                        <p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider leading-relaxed opacity-80">
                                            {t("shift_swaps.guide.step2.desc")}
                                        </p>
                                    </div>
                                    <div className="space-y-3">
                                        <div className="w-8 h-8 rounded-full bg-emerald-100 dark:bg-emerald-900/50 text-emerald-600 dark:text-emerald-400 flex items-center justify-center font-bold font-mono text-xs">
                                            3
                                        </div>
                                        <h4 className="font-bold text-sm text-slate-700 dark:text-slate-300">
                                            {t("shift_swaps.guide.step3.title")}
                                        </h4>
                                        <p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider leading-relaxed opacity-80">
                                            {t("shift_swaps.guide.step3.desc")}
                                        </p>
                                    </div>
                                </div>

                                <div className="mt-6 p-4 bg-white/50 dark:bg-slate-900/30 rounded-2xl flex items-start gap-3">
                                    <Info className="text-blue-500 mt-0.5 shrink-0" size={16} />
                                    <p className="text-xs font-bold text-blue-800 dark:text-blue-200 tracking-tight leading-relaxed">
                                        {t("shift_swaps.guide.note")}
                                    </p>
                                </div>
                            </div>
                          )}
                    </div>

                    {/* Quick Stats */}
                    {!!stats && (
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 animate-in slide-in-from-bottom-4 duration-500 delay-100">
                            {[
                                {
                                    key: "pending_admin",
                                    label: t("shift_swaps.status.pending_admin"),
                                    value: stats.pending_admin,
                                    color: "amber",
                                },
                                {
                                    key: "pending_target",
                                    label: t("shift_swaps.status.pending_target"),
                                    value: stats.pending_target,
                                    color: "blue",
                                },
                                {
                                    key: "approved",
                                    label: t("shift_swaps.status.approved"),
                                    value: stats.approved,
                                    color: "emerald",
                                },
                                {
                                    key: "rejected",
                                    label: t("shift_swaps.status.rejected"),
                                    value: stats.rejected,
                                    color: "rose",
                                },
                            ].map((stat) => (
                                <div
                                    className={cn(
                                        "bg-white dark:bg-slate-900 p-5 rounded-[2rem] border shadow-sm cursor-pointer hover:shadow-md transition-all group",
                                        activeTab === stat.key
                                            ? `border-${stat.color}-300 dark:border-${stat.color}-700 ring-2 ring-${stat.color}-100 dark:ring-${stat.color}-900`
                                            : "border-slate-100 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700",
                                    )}
                                    key={stat.key}
                                    onClick={() => { setActiveTab(stat.key as TabType); }}
                                    onKeyDown={(e) => {
                                        if (e.key === "Enter" || e.key === " ") {
                                            e.preventDefault();
                                            setActiveTab(stat.key as TabType);
                                        }
                                    }}
                                    role="button"
                                    tabIndex={0}
                                >
                                    <div className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-2 group-hover:text-slate-600 dark:group-hover:text-slate-300 transition-colors">
                                        {stat.label}
                                    </div>
                                    <div
                                        className={`text-3xl font-black tracking-tighter text-${stat.color}-600 dark:text-${stat.color}-400 font-mono`}
                                    >
                                        {stat.value}
                                    </div>
                                </div>
                            ))}
                        </div>
                      )}

                    {/* Create Form */}
                    {!!showCreateForm && (
                        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2rem] shadow-sm animate-in zoom-in-95 duration-300 mb-8">
                            <h3 className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight mb-6 flex items-center gap-2">
                                <ArrowLeftRight className="text-primary-500" />
                                {t("shift_swaps.create_title")}
                            </h3>

                            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-6">
                                {/* Requester (Employee initiating swap) */}
                                <div className="space-y-3">
                                    <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
                                        {t("shift_swaps.requester")}
                                    </label>
                                    <Combobox
                                        className="h-14 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-700"
                                        onChange={setRequesterId}
                                        options={employees.map((e) => ({
                                            value: e.id,
                                            label: e.full_name.toUpperCase(),
                                        }))}
                                        placeholder={t("common.employee")}
                                        value={requesterId}
                                    />
                                </div>

                                {/* Target (Employee covering shift) */}
                                <div className="space-y-3">
                                    <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
                                        {t("shift_swaps.target")}
                                    </label>
                                    <Combobox
                                        className="h-14 bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-700"
                                        disabled={!requesterId}
                                        onChange={setTargetId}
                                        options={employees
                                            .filter((e) => e.id !== requesterId)
                                            .map((e) => ({
                                                value: e.id,
                                                label: e.full_name.toUpperCase(),
                                            }))}
                                        placeholder={t("common.employee")}
                                        value={targetId}
                                    />
                                </div>
                            </div>

                            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
                                <FormField
                                    id="swap_date"
                                    inputClassName="text-slate-700 dark:text-slate-300 border-slate-100 dark:border-slate-800"
                                    label={t("shift_swaps.swap_date")}
                                    name="swap_date"
                                    onChange={(e) => { setSwapDate(e.target.value); }}
                                    required
                                    type="date"
                                    value={swapDate}
                                />

                                <FormField
                                    id="swap_reason"
                                    inputClassName="text-slate-700 dark:text-slate-300 border-slate-100 dark:border-slate-800"
                                    label={t("shift_swaps.reason")}
                                    name="reason"
                                    onChange={(e) => { setReason(e.target.value); }}
                                    placeholder={t("common.no_notes")}
                                    type="text"
                                    value={reason}
                                />
                            </div>

                            <div className="flex justify-end gap-4 pt-6 border-t border-slate-50 dark:border-slate-800">
                                <Button
                                    onClick={() => { setShowCreateForm(false); }}
                                    variant="outline"
                                >
                                    {t("common.cancel")}
                                </Button>
                                <Button
                                    disabled={
                                        createMutation.isPending ||
                                        !requesterId ||
                                        !targetId ||
                                        !swapDate
                                    }
                                    onClick={handleCreate}
                                >
                                    {t("common.save")}
                                </Button>
                            </div>
                        </div>
                      )}
                </>
            }
            emptyMessage={t("shift_swaps.messages.no_data")}
            headerActions={
                <Button icon={<Plus size={18} />} onClick={() => { setShowCreateForm(!showCreateForm); }}>
                    {t("shift_swaps.create_btn")}
                </Button>
            }
            isEmpty={swaps.length === 0}
            isLoading={isLoading}
            onTabChange={(id) => { setActiveTab(id as TabType); }}
            subtitle={t("shift_swaps.subtitle")}
            tableHeaders={tableHeaders}
            tabs={tabs}
            title={t("shift_swaps.title")}
        >
            {swaps.map((swap) => (
                <tr
                    className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group"
                    key={swap.id}
                >
                    {/* Date */}
                    <td className="px-10 py-6">
                        <div className="inline-flex items-center gap-2 bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-inner">
                            <CalendarDays
                                className="text-primary-600 dark:text-primary-400"
                                size={14}
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
                            className="text-slate-300 dark:text-slate-600 mx-auto"
                            size={16}
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
                        <StatusBadge variant={
                            swap.status === "pending_admin" ? "warning" :
                            swap.status === "approved" ? "success" : "danger"
                        }>
                            {t(`shift_swaps.status.${swap.status}`)}
                        </StatusBadge>
                    </td>

                    {/* Actions */}
                    <td className="px-10 py-6 text-right">
                        {swap.status === "pending_admin" && (
                            <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">
                                <button
                                    className="p-3 bg-emerald-600 text-white hover:bg-emerald-700 rounded-xl shadow-lg shadow-emerald-100 transition-all active:scale-90"
                                    onClick={() =>
                                        { approveMutation.mutate({
                                            id: swap.id,
                                            status: "approved",
                                        }); }
                                    }
                                    title={t("common.approve")}
                                >
                                    <Check size={18} />
                                </button>
                                <button
                                    className="p-3 bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 rounded-xl shadow-md border border-rose-100 dark:border-rose-800 transition-all active:scale-90"
                                    onClick={() =>
                                        { approveMutation.mutate({
                                            id: swap.id,
                                            status: "rejected",
                                        }); }
                                    }
                                    title={t("common.reject")}
                                >
                                    <X size={18} />
                                </button>
                            </div>
                        )}
                    </td>
                </tr>
            ))}
        </ApprovalListLayout>
    );
}
