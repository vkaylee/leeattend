import React, { useState, useMemo, useEffect, useRef } from "react";

import { useQuery, useMutation, useQueryClient, useInfiniteQuery } from "@tanstack/react-query";
import { CalendarCheck, SearchX, FileBarChart, Clock  } from "lucide-react";
import { useTranslation } from "react-i18next";

import InfiniteScrollSentinel from "@/components/common/InfiniteScrollSentinel";
import Sheet from "@/components/ui/Sheet";
import { useToast } from "@/components/ui/Toast";


import { useDebug } from "@/context/DebugContext";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
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
    fetchShifts,
    upsertRoster,
} from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";

import ConfirmAttendanceModal from "./ConfirmAttendanceModal";
import ConfirmReprocessModal from "./ConfirmReprocessModal";
import DailyReportTable from "./DailyReportTable";
import ResolveUnscheduledModal from "./ResolveUnscheduledModal";

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
    const contentRef = useRef<HTMLDivElement>(null);

    // Reset to daily tab whenever the drawer is (re-)opened
    // eslint-disable-next-line react-hooks/set-state-in-effect -- intentional: reset derived state on prop change
    useEffect(() => { if (isOpen) setViewMode("daily"); }, [isOpen]);

    // Keyboard navigation for tabs + scroll
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            // Do not handle if typing in an input
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
                return;
            }

            if (e.key === "ArrowRight" && viewMode === "daily") {
                setViewMode("raw");
            } else if (e.key === "ArrowLeft" && viewMode === "raw") {
                setViewMode("daily");
            } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                e.preventDefault();
                const scrollContainer = contentRef.current?.querySelector(".overflow-auto") || contentRef.current;
                if (scrollContainer) {
                    scrollContainer.scrollBy({ top: e.key === "ArrowDown" ? 100 : -100, behavior: "smooth" });
                }
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [isOpen, viewMode]);

    const [expandedDebugRows, setExpandedDebugRows] = useState<Set<string>>(new Set());
    const [reprocessingRow, setReprocessingRow] = useState<string | null>(null);
    const [confirmingRow, setConfirmingRow] = useState<string | null>(null);
    const [confirmDialog, setConfirmDialog] = useState<{
        show: boolean;
        employeeId: string;
        employeeName: string;
        date: string;
    } | null>(null);
    const [confirmModalData, setConfirmModalData] = useState<{
        row: DailyAttendanceReport;
        existingRequest?: AttendanceAdjustmentRequest;
    } | null>(null);
    const [resolveModalData, setResolveModalData] = useState<{
        row: DailyAttendanceReport;
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

    // Fetch shift types for the unscheduled resolution modal
    const { data: shifts } = useQuery({
        queryKey: ["shifts", workspaceId],
        queryFn: () => fetchShifts(workspaceId),
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

    const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
        hasNextPage,
        isFetchingNextPage,
        fetchNextPage,
    });

    const adjustmentMap = useMemo(() => {
        const map: Record<string, AttendanceAdjustmentRequest> = {};
        pendingAdjustments?.forEach((req) => {
            const key = `${req.employee_id}-${req.requested_date}`;
            map[key] = req;
        });
        return map;
    }, [pendingAdjustments]);

    const reportData = useMemo(
        () => (reportResponse?.data || (Array.isArray(reportResponse) ? reportResponse : [])) as DailyAttendanceReport[],
        [reportResponse]
    );

    // Compute total hours for summary bar
    const { totalStdHours, totalOtHours } = useMemo(() => {
        const stdUnits = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.std_work_units || 0), 0);
        const otUnits = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.ot_work_units || 0), 0);
        return {
            totalStdHours: formatUnitsToHours(stdUnits, workUnitMinutes),
            totalOtHours: formatUnitsToHours(otUnits, workUnitMinutes),
        };
    }, [reportData, workUnitMinutes]);

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
            setConfirmModalData(null);
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

    // Resolve unscheduled mutation: upsert roster + reprocess
    const resolveUnscheduledMutation = useMutation({
        mutationFn: async ({ row, shiftId }: { row: DailyAttendanceReport; shiftId: string }) => {
            // Step 1: Assign the shift to this day
            await upsertRoster(workspaceId, row.employee_id, {
                date: row.date,
                shift_id: shiftId,
            });
            // Step 2: Reprocess to recalculate attendance with the new shift
            await reprocessAttendance(workspaceId, row.employee_id, row.date, row.date);
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            setResolveModalData(null);
            toast.success(t("reports.unscheduled.success"));
        },
        onError: () => {
            toast.error(t("reports.unscheduled.error"));
        },
    });

    const handleApproveOt = (row: DailyAttendanceReport, action: "approved" | "rejected") => {
        approveOtMutation.mutate({ row, action });
    };

    const handleResolveUnscheduled = (row: DailyAttendanceReport) => {
        setResolveModalData({ row });
    };

    const handleResolveSubmit = (shiftId: string) => {
        if (!resolveModalData) return;
        resolveUnscheduledMutation.mutate({
            row: resolveModalData.row,
            shiftId,
        });
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
        setConfirmModalData({ row, existingRequest: existing });
    };

    const handleConfirmSubmit = ({ punchIn, punchOut, reason }: { punchIn: string; punchOut: string; reason: string }) => {
        if (!confirmModalData) return;
        const { row, existingRequest } = confirmModalData;
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
                actual_punch_in: createIsoFromLocal(row.date, punchIn),
                actual_punch_out: createIsoFromLocal(row.date, punchOut),
            };
            confirmMutation.mutate({ row: modifiedRow as DailyAttendanceReport, reason });
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
                contentClassName="!overflow-hidden"
                headerExtra={
                    <>
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
                        {viewMode === "daily" && !isLoading && reportData.length > 0 && (
                            <>
                                <div className="flex items-center gap-1.5 bg-slate-900 dark:bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-800 dark:border-slate-700 shadow-sm">
                                    <FileBarChart size={12} className="text-primary-400" />
                                    <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest">{t("reports.quick_stats.std_hours")}</span>
                                    <span className="font-mono font-bold text-white text-xs tracking-tighter">{totalStdHours}h</span>
                                </div>
                                {Number(totalOtHours) > 0 && (
                                    <div className="flex items-center gap-1.5 bg-gradient-to-r from-orange-900 to-orange-950 px-3 py-1.5 rounded-lg border border-orange-800 shadow-sm">
                                        <Clock size={12} className="text-orange-400" />
                                        <span className="text-[9px] font-bold text-orange-400/70 uppercase tracking-widest">{t("reports.quick_stats.ot_hours")}</span>
                                        <span className="font-mono font-bold text-orange-100 text-xs tracking-tighter">{totalOtHours}h</span>
                                    </div>
                                )}
                            </>
                        )}
                    </>
                }
            >
                <div ref={contentRef} className="-mx-10 -mt-4 -mb-10 h-[calc(100%+theme(spacing.14))] flex flex-col">
                    {viewMode === "raw" ? (
                        <div className="overflow-auto flex-1 min-h-0 relative">
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
                                                            {log.device_name && (
                                                                <span className="text-[10px] font-bold text-slate-700 dark:text-slate-200 uppercase">{log.device_name}</span>
                                                            )}
                                                        </div>
                                                    </div>
                                                </td>
                                                <td className="px-8 py-5 text-right">
                                                    <span className="px-3 py-1 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 rounded-full text-[9px] font-bold uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm">{t("attendance_logs.status.recorded")}</span>
                                                </td>
                                            </tr>
                                        ))
                                    )}
                                    {hasNextPage && (
                                        <InfiniteScrollSentinel sentinelRef={loadMoreRef} isFetchingNextPage={isFetchingNextPage} colSpan={3} />
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
                            onResolveUnscheduled={handleResolveUnscheduled}
                            hideEmployeeName
                        />
                    )}
                </div>
            </Sheet>

            {/* Extracted Modals */}
            <ConfirmReprocessModal
                key={confirmDialog ? `reprocess-${confirmDialog.employeeId}-${confirmDialog.date}` : "reprocess-empty"}
                isOpen={!!confirmDialog}
                onClose={() => setConfirmDialog(null)}
                dialogData={confirmDialog}
                isPending={reprocessMutation.isPending}
                onConfirm={handleConfirmReprocess}
            />

            <ConfirmAttendanceModal
                key={confirmModalData ? `confirm-${confirmModalData.row.date}` : "confirm-empty"}
                isOpen={!!confirmModalData}
                onClose={() => setConfirmModalData(null)}
                row={confirmModalData?.row || null}
                existingRequest={confirmModalData?.existingRequest}
                isPending={confirmMutation.isPending}
                onSubmit={handleConfirmSubmit}
            />

            <ResolveUnscheduledModal
                key={resolveModalData ? `resolve-${resolveModalData.row.date}` : "resolve-empty"}
                isOpen={!!resolveModalData}
                onClose={() => setResolveModalData(null)}
                row={resolveModalData?.row || null}
                shifts={shifts}
                isPending={resolveUnscheduledMutation.isPending}
                onSubmit={handleResolveSubmit}
            />
        </>
    );
}
