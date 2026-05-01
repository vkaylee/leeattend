import React, { useRef, useEffect } from "react";

import { CalendarCheck, FileBarChart, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import Sheet from "@/components/ui/Sheet";

import { useDebug } from "@/context/DebugContext";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

import ConfirmAttendanceModal from "./ConfirmAttendanceModal";
import ConfirmReprocessModal from "./ConfirmReprocessModal";
import DailyReportTable from "./DailyReportTable";
import RawLogsTable from "./RawLogsTable";
import ResolveUnscheduledModal from "./ResolveUnscheduledModal";
import { useEmployeeDailyDrawer } from "./useEmployeeDailyDrawer";

interface EmployeeDailyDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    workspaceId: string;
    employeeId: string;
    employeeName: string;
    dateRange: { start: string; end: string };
    workUnitMinutes: number;
}

export default function EmployeeDailyDrawer(props: EmployeeDailyDrawerProps) {
    const { isOpen, onClose, employeeName, dateRange, workUnitMinutes } = props;
    const { t } = useTranslation();
    const { formatDate, formatTime } = useDateTimeFormatter();
    const { isEnabled: isDebugMode } = useDebug();
    const contentRef = useRef<HTMLDivElement>(null);

    const {
        viewMode,
        setViewMode,
        expandedDebugRows,
        toggleExpandRow,
        reprocessingRow,
        confirmingRow,
        confirmDialog,
        setConfirmDialog,
        confirmModalData,
        setConfirmModalData,
        resolveModalData,
        setResolveModalData,
        reportData,
        isLoading,
        rawLogsData,
        rawLogsLoading,
        hasNextPage,
        isFetchingNextPage,
        fetchNextPage,
        adjustmentMap,
        shifts,
        totalStdHours,
        totalOtHours,
        handleReprocessClick,
        handleConfirmReprocess,
        handleConfirmClick,
        handleConfirmSubmit,
        handleApproveOt,
        handleResolveUnscheduled,
        handleResolveSubmit,
        reprocessMutationPending,
        confirmMutationPending,
        resolveUnscheduledMutationPending,
    } = useEmployeeDailyDrawer(props);

    // Keyboard navigation for tabs + scroll
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
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
    }, [isOpen, viewMode, setViewMode]);

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
                        <RawLogsTable
                            rawLogsData={rawLogsData}
                            rawLogsLoading={rawLogsLoading}
                            hasNextPage={hasNextPage}
                            isFetchingNextPage={isFetchingNextPage}
                            fetchNextPage={fetchNextPage}
                        />
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

            <ConfirmReprocessModal
                key={confirmDialog ? `reprocess-${confirmDialog.employeeId}-${confirmDialog.date}` : "reprocess-empty"}
                isOpen={!!confirmDialog}
                onClose={() => setConfirmDialog(null)}
                dialogData={confirmDialog}
                isPending={reprocessMutationPending}
                onConfirm={handleConfirmReprocess}
            />

            <ConfirmAttendanceModal
                key={confirmModalData ? `confirm-${confirmModalData.row.date}` : "confirm-empty"}
                isOpen={!!confirmModalData}
                onClose={() => setConfirmModalData(null)}
                row={confirmModalData?.row || null}
                existingRequest={confirmModalData?.existingRequest}
                isPending={confirmMutationPending}
                onSubmit={handleConfirmSubmit}
            />

            <ResolveUnscheduledModal
                key={resolveModalData ? `resolve-${resolveModalData.row.date}` : "resolve-empty"}
                isOpen={!!resolveModalData}
                onClose={() => setResolveModalData(null)}
                row={resolveModalData?.row || null}
                shifts={shifts}
                isPending={resolveUnscheduledMutationPending}
                onSubmit={handleResolveSubmit}
            />
        </>
    );
}
