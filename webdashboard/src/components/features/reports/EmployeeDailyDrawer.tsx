import React, { useState, useMemo, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient, useInfiniteQuery } from "@tanstack/react-query";
import { RefreshCw, Check, CheckCircle, User, CalendarCheck, SearchX, Loader2 } from "lucide-react";
import { Link } from "react-router-dom";
import {
    fetchAttendanceReport,
    DailyAttendanceReport,
    reprocessAttendance,
    fetchWorkspaceAttendanceAdjustments,
    adminConfirmAttendance,
    AttendanceAdjustmentRequest,
    approveOt,
    fetchRawAttendanceLogs,
    RawAttendanceLog,
} from "@/lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDebug } from "@/context/DebugContext";
import { useToast } from "@/components/ui/Toast";
import Sheet from "@/components/ui/Sheet";
import Modal from "@/components/ui/Modal";
import DailyReportTable from "./DailyReportTable";
import { Clock, HardDrive } from "lucide-react";

interface EmployeeDailyDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    workspaceId: string;
    employeeId: string;
    employeeName: string;
    dateRange: { start: string; end: string };
    workUnitMinutes: number;
}

export default function EmployeeDailyDrawer({
    isOpen,
    onClose,
    workspaceId,
    employeeId,
    employeeName,
    dateRange,
    workUnitMinutes,
}: EmployeeDailyDrawerProps) {
    const { t } = useTranslation();
    const { formatDate, formatTime } = useDateTimeFormatter();
    const queryClient = useQueryClient();
    const { isEnabled: isDebugMode } = useDebug();
    const toast = useToast();

    const [viewMode, setViewMode] = useState<"daily" | "raw">("daily");

    const [expandedDebugRows, setExpandedDebugRows] = useState<Set<string>>(new Set());
    const [reprocessingRow, setReprocessingRow] = useState<string | null>(null);
    const [confirmingRow, setConfirmingRow] = useState<string | null>(null);
    const [confirmDialog, setConfirmDialog] = useState<{
        show: boolean;
        employeeId: string;
        employeeName: string;
        date: string;
    } | null>(null);
    const [confirmModal, setConfirmModal] = useState<{
        row: DailyAttendanceReport;
        existingRequest?: AttendanceAdjustmentRequest;
        punchIn: string;
        punchOut: string;
        reason: string;
    } | null>(null);

    // Fetch daily data for this single employee
    const { data: reportResponse, isLoading } = useQuery({
        queryKey: ["attendance-report", workspaceId, dateRange, "daily", 1, 100, undefined, [employeeId]],
        queryFn: () =>
            fetchAttendanceReport(workspaceId, dateRange.start, dateRange.end, "daily", 1, 100, undefined, [employeeId]),
        enabled: isOpen && !!workspaceId && !!employeeId,
    });

    const { data: pendingAdjustments } = useQuery({
        queryKey: ["attendance-adjustments", workspaceId, "pending"],
        queryFn: () => fetchWorkspaceAttendanceAdjustments(workspaceId, "pending"),
        enabled: isOpen && !!workspaceId,
    });

    // Fetch raw logs for this single employee with infinite scroll
    const {
        data: rawLogsResponse,
        isLoading: rawLogsLoading,
        fetchNextPage,
        hasNextPage,
        isFetchingNextPage
    } = useInfiniteQuery({
        queryKey: ["raw-logs", workspaceId, dateRange, [employeeId]],
        queryFn: ({ pageParam }) => {
            const from = dateRange.start ? new Date(`${dateRange.start}T00:00:00`).toISOString() : undefined;
            const to = dateRange.end ? new Date(`${dateRange.end}T23:59:59`).toISOString() : undefined;
            return fetchRawAttendanceLogs({ employee_id: employeeId, from, to, page: pageParam as number, limit: 100 });
        },
        getNextPageParam: (lastPage, allPages) => {
            if (!lastPage.data || lastPage.data.length < 100) return undefined;
            return allPages.length + 1;
        },
        initialPageParam: 1,
        enabled: isOpen && !!workspaceId && !!employeeId && viewMode === "raw",
    });

    // Extract flat array of logs
    const rawLogsData = useMemo(() => {
        if (!rawLogsResponse) return [];
        return rawLogsResponse.pages.flatMap((p) => p.data || []);
    }, [rawLogsResponse]);

    // Intersection observer for infinite scroll
    const loadMoreRef = useRef<HTMLTableRowElement>(null);
    useEffect(() => {
        if (!loadMoreRef.current || !hasNextPage) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0].isIntersecting && !isFetchingNextPage) {
                    fetchNextPage();
                }
            },
            { threshold: 0.1, rootMargin: "100px" }
        );
        observer.observe(loadMoreRef.current);
        return () => observer.disconnect();
    }, [fetchNextPage, hasNextPage, isFetchingNextPage]);

    const adjustmentMap = useMemo(() => {
        const map: Record<string, AttendanceAdjustmentRequest> = {};
        pendingAdjustments?.forEach((req) => {
            const key = `${req.employee_id}-${req.requested_date}`;
            map[key] = req;
        });
        return map;
    }, [pendingAdjustments]);

    const reportData = reportResponse?.data || (Array.isArray(reportResponse) ? reportResponse : []);

    const toggleExpandRow = (rowKey: string) => {
        const next = new Set(expandedDebugRows);
        if (next.has(rowKey)) next.delete(rowKey);
        else next.add(rowKey);
        setExpandedDebugRows(next);
    };

    // Reprocess mutation
    const reprocessMutation = useMutation({
        mutationFn: ({ empId, date }: { empId: string; date: string }) =>
            reprocessAttendance(workspaceId, empId, date, date),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            setReprocessingRow(null);
            setConfirmDialog(null);
            toast.success(t("common.success"));
        },
    });

    // Confirm mutation
    const confirmMutation = useMutation({
        mutationFn: ({
            row,
            existingRequestId,
            reason,
        }: {
            row: DailyAttendanceReport;
            existingRequestId?: string;
            reason?: string;
        }) => adminConfirmAttendance(workspaceId, row, existingRequestId, reason),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] });
            setConfirmingRow(null);
            setConfirmModal(null);
            toast.success(t("reports.confirm.success"));
        },
        onError: () => {
            setConfirmingRow(null);
            toast.error(t("reports.confirm.error"));
        },
    });

    // OT approve mutation
    const approveOtMutation = useMutation({
        mutationFn: ({ row, action }: { row: DailyAttendanceReport; action: "approved" | "rejected" }) =>
            approveOt(workspaceId, row.employee_id, {
                date: row.date,
                approved_minutes: row.ot_work_units * (workUnitMinutes || 60),
                status: action,
            }),
        onSuccess: (_data, variables) => {
            queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            const msg =
                variables.action === "approved"
                    ? t("reports.ot_approve.success_approved")
                    : t("reports.ot_approve.success_rejected");
            toast.success(msg);
        },
        onError: () => {
            toast.error(t("reports.ot_approve.error"));
        },
    });

    const handleApproveOt = (row: DailyAttendanceReport, action: "approved" | "rejected") => {
        approveOtMutation.mutate({ row, action });
    };

    const handleReprocessClick = (row: DailyAttendanceReport) => {
        setConfirmDialog({
            show: true,
            employeeId: row.employee_id,
            employeeName: row.employee_name,
            date: row.date,
        });
    };

    const handleConfirmReprocess = () => {
        if (!confirmDialog) return;
        setReprocessingRow(`${confirmDialog.employeeId}-${confirmDialog.date}`);
        reprocessMutation.mutate({ empId: confirmDialog.employeeId, date: confirmDialog.date });
    };

    const handleConfirmClick = (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => {
        const utcToLocalHHmm = (iso: string | null | undefined): string => {
            if (!iso) return "";
            const d = new Date(iso);
            if (isNaN(d.getTime())) {
                return iso.substring(0, 5);
            }
            return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
        };

        const punchIn =
            utcToLocalHHmm(existing?.requested_punch_in_time) ||
            utcToLocalHHmm(row.actual_punch_in) ||
            row.scheduled_start?.substring(0, 5) ||
            "";
        const punchOut =
            utcToLocalHHmm(existing?.requested_punch_out_time) ||
            utcToLocalHHmm(row.actual_punch_out) ||
            row.scheduled_end?.substring(0, 5) ||
            "";
        const reason = existing?.reason || "";
        setConfirmModal({ row, existingRequest: existing, punchIn, punchOut, reason });
    };

    const handleConfirmSubmit = () => {
        if (!confirmModal) return;
        const { row, existingRequest } = confirmModal;
        const rowKey = `${row.employee_id}-${row.date}`;
        setConfirmingRow(rowKey);

        if (existingRequest) {
            confirmMutation.mutate({ row, existingRequestId: existingRequest.id });
        } else {
            const createIsoFromLocal = (dateStr: string, hhmm: string): string | null => {
                if (!hhmm) return null;
                const [y, m, d] = dateStr.split("-").map(Number);
                const [hours, mins] = hhmm.split(":").map(Number);
                return new Date(y, m - 1, d, hours, mins, 0).toISOString();
            };

            const modifiedRow = {
                ...row,
                actual_punch_in: createIsoFromLocal(row.date, confirmModal.punchIn),
                actual_punch_out: createIsoFromLocal(row.date, confirmModal.punchOut),
            };
            confirmMutation.mutate({ row: modifiedRow as DailyAttendanceReport, reason: confirmModal.reason });
        }
    };

    return (
        <>
            <Sheet
                isOpen={isOpen}
                onClose={onClose}
                title={employeeName}
                subtitle={`${formatDate(dateRange.start)} — ${formatDate(dateRange.end)}`}
                className="w-[95vw] xl:w-[1200px] !max-w-[95vw]"
                headerExtra={
                    <div className="flex bg-slate-100 dark:bg-slate-800 rounded-lg p-1 border border-slate-200 dark:border-slate-700">
                        <button
                            onClick={() => setViewMode("daily")}
                            className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all ${viewMode === "daily"
                                ? "bg-white dark:bg-slate-700 text-primary-600 dark:text-primary-400 shadow-sm"
                                : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
                                }`}
                        >
                            {t("reports.tabs.daily")}
                        </button>
                        <button
                            onClick={() => setViewMode("raw")}
                            className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all ${viewMode === "raw"
                                ? "bg-white dark:bg-slate-700 text-primary-600 dark:text-primary-400 shadow-sm"
                                : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
                                }`}
                        >
                            <span className="flex items-center gap-1.5"><CalendarCheck size={12} /> {t("reports.view_raw_logs")}</span>
                        </button>
                    </div>
                }
            >
                <div className="-mx-10 -mt-4">
                    {viewMode === "raw" ? (
                        <div className="overflow-auto max-h-[calc(100vh-250px)] relative">
                            <table className="w-full text-left border-collapse">
                                <thead>
                                    <tr className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky top-0 z-10">
                                        <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.time")}</th>
                                        <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.source")}</th>
                                        <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.status")}</th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                                    {rawLogsLoading ? (
                                        Array.from({ length: 5 }).map((_, i) => (
                                            <tr key={`skeleton-log-${i}`} className="border-b border-slate-50 dark:border-slate-800/50">
                                                <td className="px-8 py-5">
                                                    <div className="flex items-center gap-4">
                                                        <div className="w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-800 animate-pulse"></div>
                                                        <div className="space-y-1">
                                                            <div className="h-4 w-20 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div>
                                                            <div className="h-3 w-16 bg-slate-100 dark:bg-slate-800 animate-pulse rounded"></div>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td className="px-8 py-5">
                                                    <div className="h-4 w-24 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div>
                                                </td>
                                                <td className="px-8 py-5 text-right">
                                                    <div className="flex justify-end">
                                                        <div className="h-4 w-16 bg-emerald-100 dark:bg-emerald-900/20 animate-pulse rounded-full"></div>
                                                    </div>
                                                </td>
                                            </tr>
                                        ))
                                    ) : rawLogsData.length === 0 ? (
                                        <tr>
                                            <td colSpan={3} className="px-8 py-20">
                                                <div className="flex flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500 max-w-sm mx-auto">
                                                    <div className="w-16 h-16 mb-4 rounded-3xl bg-slate-50 dark:bg-slate-800/80 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner border border-slate-100 dark:border-slate-700/50 rotate-3">
                                                        <SearchX size={24} className="-rotate-3" />
                                                    </div>
                                                    <h3 className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest mb-1">
                                                        {t("attendance_logs.messages.no_data")}
                                                    </h3>
                                                    <p className="text-[10px] font-medium text-slate-500 dark:text-slate-400 max-w-[200px]">
                                                        No punches were recorded for this day.
                                                    </p>
                                                </div>
                                            </td>
                                        </tr>
                                    ) : (
                                        rawLogsData.map((log: RawAttendanceLog) => (
                                            <tr key={log.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group">
                                                <td className="px-8 py-5">
                                                    <div className="flex items-center gap-4">
                                                        <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50"><Clock size={18} /></div>
                                                        <div>
                                                            <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
                                                            <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td className="px-8 py-5">
                                                    <div className="flex flex-col gap-1">
                                                        <div className="flex items-center gap-2">
                                                            <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700">{log.source}</span>
                                                            {(log.device_name || log.device_id) && (
                                                                <span className="text-[10px] font-bold text-slate-700 dark:text-slate-200 uppercase">{log.device_name || log.device_id}</span>
                                                            )}
                                                        </div>
                                                        {log.device_id && (
                                                            <div className="text-[8px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-tighter flex items-center gap-1 ml-0.5">
                                                                <HardDrive size={10} /> {log.device_id}
                                                            </div>
                                                        )}
                                                    </div>
                                                </td>
                                                <td className="px-8 py-5 text-right">
                                                    <span className="px-3 py-1 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 rounded-full text-[9px] font-bold uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm">{t("attendance_logs.status.recorded")}</span>
                                                </td>
                                            </tr>
                                        ))
                                    )}
                                    {hasNextPage && (
                                        <tr ref={loadMoreRef}>
                                            <td colSpan={3} className="px-8 py-6 text-center">
                                                {isFetchingNextPage && (
                                                    <Loader2 className="w-5 h-5 animate-spin mx-auto text-slate-400 dark:text-slate-500" />
                                                )}
                                            </td>
                                        </tr>
                                    )}
                                </tbody>
                            </table>
                        </div>
                    ) : (
                        <DailyReportTable
                            data={reportData}
                            isLoading={isLoading}
                            workUnitMinutes={workUnitMinutes}
                            isDebugMode={isDebugMode}
                            expandedDebugRows={expandedDebugRows}
                            reprocessingRow={reprocessingRow}
                            confirmingRow={confirmingRow}
                            adjustmentMap={adjustmentMap}
                            t={t}
                            formatDate={formatDate}
                            formatTime={formatTime}
                            onToggleExpandRow={toggleExpandRow}
                            onReprocessClick={handleReprocessClick}
                            onConfirmClick={handleConfirmClick}
                            onApproveOt={handleApproveOt}
                            hideEmployeeName
                        />
                    )}
                </div>
            </Sheet>

            {/* Reprocess Confirmation Modal */}
            <Modal
                isOpen={!!confirmDialog}
                onClose={() => !reprocessMutation.isPending && setConfirmDialog(null)}
                title={t("reports.reprocess.confirm_title")}
                icon={<RefreshCw size={24} />}
                maxWidth="md"
            >
                <div className="space-y-10 pt-4 animate-in fade-in duration-500">
                    <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
                        <p className="text-base font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight leading-relaxed text-center italic opacity-80">
                            "
                            {confirmDialog &&
                                t("reports.reprocess.confirm_message", {
                                    employee: confirmDialog.employeeName,
                                    date: formatDate(confirmDialog.date),
                                })}
                            "
                        </p>
                    </div>
                    <div className="flex gap-6 border-t border-slate-50 dark:border-slate-800 pt-2">
                        <button
                            onClick={() => setConfirmDialog(null)}
                            className="flex-1 px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
                            disabled={reprocessMutation.isPending}
                        >
                            {t("common.cancel")}
                        </button>
                        <button
                            onClick={handleConfirmReprocess}
                            disabled={reprocessMutation.isPending}
                            className="flex-[1.5] bg-slate-900 hover:bg-primary-700 text-white font-bold px-8 py-4 rounded-2xl flex items-center justify-center gap-4 transition-all active:scale-95 disabled:opacity-50 shadow-2xl shadow-slate-400 dark:shadow-black/40 text-xs uppercase tracking-widest"
                        >
                            {reprocessMutation.isPending ? (
                                <RefreshCw className="w-5 h-5 animate-spin" />
                            ) : (
                                <Check size={20} />
                            )}{" "}
                            {t("reports.reprocess.confirm").toUpperCase()}
                        </button>
                    </div>
                </div>
            </Modal>

            {/* Confirm Attendance Modal */}
            <Modal
                isOpen={!!confirmModal}
                onClose={() => !confirmMutation.isPending && setConfirmModal(null)}
                title={t("reports.confirm.modal_title")}
                subtitle={t("reports.confirm.modal_subtitle")}
                icon={<CheckCircle size={24} />}
                maxWidth="md"
            >
                {confirmModal && (
                    <div className="space-y-6 pt-2 animate-in fade-in duration-500">
                        {/* Employee & Date info */}
                        <div className="grid grid-cols-2 gap-4 p-5 bg-slate-50 dark:bg-slate-800 rounded-2xl border border-slate-100 dark:border-slate-800">
                            <div>
                                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                                    {t("reports.confirm.employee")}
                                </label>
                                <p className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1">
                                    {confirmModal.row.employee_name}
                                </p>
                            </div>
                            <div>
                                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                                    {t("reports.confirm.date")}
                                </label>
                                <p className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1">
                                    {formatDate(confirmModal.row.date)}
                                </p>
                            </div>
                        </div>

                        {/* Existing request banner */}
                        {confirmModal.existingRequest && (
                            <div className="p-4 bg-orange-50 dark:bg-orange-950 rounded-2xl border border-orange-200 dark:border-orange-800">
                                <p className="text-xs font-bold text-orange-600 dark:text-orange-300 uppercase tracking-wider">
                                    {t("reports.confirm.existing_request")}
                                </p>
                                {confirmModal.existingRequest.reason && (
                                    <p className="text-sm text-orange-700 dark:text-orange-200 mt-2 italic">
                                        &ldquo;{confirmModal.existingRequest.reason}&rdquo;
                                    </p>
                                )}
                            </div>
                        )}

                        {/* Punch In / Out */}
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
                                    {t("reports.confirm.punch_in")}
                                </label>
                                <input
                                    type="time"
                                    value={confirmModal.punchIn}
                                    onChange={(e) => setConfirmModal({ ...confirmModal, punchIn: e.target.value })}
                                    disabled={!!confirmModal.existingRequest}
                                    className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-mono font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                                />
                            </div>
                            <div>
                                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
                                    {t("reports.confirm.punch_out")}
                                </label>
                                <input
                                    type="time"
                                    value={confirmModal.punchOut}
                                    onChange={(e) => setConfirmModal({ ...confirmModal, punchOut: e.target.value })}
                                    disabled={!!confirmModal.existingRequest}
                                    className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-mono font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                                />
                            </div>
                        </div>

                        {/* Reason */}
                        {!confirmModal.existingRequest && (
                            <div>
                                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
                                    {t("reports.confirm.reason")}
                                </label>
                                <input
                                    type="text"
                                    value={confirmModal.reason}
                                    onChange={(e) => setConfirmModal({ ...confirmModal, reason: e.target.value })}
                                    placeholder={t("reports.confirm.reason_placeholder")}
                                    className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                                />
                            </div>
                        )}

                        {/* Buttons */}
                        <div className="flex gap-4 border-t border-slate-100 dark:border-slate-800 pt-4">
                            <button
                                onClick={() => setConfirmModal(null)}
                                className="flex-1 px-6 py-3.5 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
                                disabled={confirmMutation.isPending}
                            >
                                {t("common.cancel")}
                            </button>
                            <button
                                onClick={handleConfirmSubmit}
                                disabled={confirmMutation.isPending || (!confirmModal.existingRequest && !confirmModal.punchIn)}
                                className="flex-[1.5] bg-emerald-600 hover:bg-emerald-700 text-white font-bold px-6 py-3.5 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-xl text-xs uppercase tracking-widest"
                            >
                                {confirmMutation.isPending ? (
                                    <RefreshCw className="w-4 h-4 animate-spin" />
                                ) : (
                                    <CheckCircle size={18} />
                                )}
                                {confirmModal.existingRequest ? t("reports.confirm.approve") : t("reports.confirm.submit")}
                            </button>
                        </div>
                    </div>
                )}
            </Modal>
        </>
    );
}
