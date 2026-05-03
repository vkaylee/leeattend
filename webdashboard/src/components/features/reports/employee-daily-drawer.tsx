import { CalendarCheck, FileBarChart, Clock } from "lucide-react";
import React, { useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";

import Sheet from "@/components/ui/sheet";
import { useDebug } from "@/context/debug-context";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

import ConfirmAttendanceModal from "./confirm-attendance-modal";
import ConfirmReprocessModal from "./confirm-reprocess-modal";
import DailyReportTable from "./daily-report-table";
import RawLogsTable from "./raw-logs-table";
import ResolveUnscheduledModal from "./resolve-unscheduled-modal";
import { useEmployeeDailyDrawer } from "./use-employee-daily-drawer";

interface EmployeeDailyDrawerProps {
    dateRange: { start: string; end: string };
    employeeId: string;
    employeeName: string;
    isOpen: boolean;
    onClose: () => void;
    workspaceId: string;
    workUnitMinutes: number;
}

function EmployeeDailyDrawerHeader({ viewMode, setViewMode, isLoading, reportData, totalStdHours, totalOtHours, t }: any) {
    return (
        <>
            <div className="flex bg-slate-100 dark:bg-slate-800 rounded-lg p-1 border border-slate-200 dark:border-slate-700">
                <button
                    className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all ${viewMode === "daily"
                        ? "bg-white dark:bg-slate-700 text-primary-600 dark:text-primary-400 shadow-sm"
                        : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
                        }`}
                    onClick={() => { setViewMode("daily"); }}
                >
                    {t("reports.tabs.daily")}
                </button>
                <button
                    className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all ${viewMode === "raw"
                        ? "bg-white dark:bg-slate-700 text-primary-600 dark:text-primary-400 shadow-sm"
                        : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
                        }`}
                    onClick={() => { setViewMode("raw"); }}
                >
                    <span className="flex items-center gap-1.5"><CalendarCheck size={12} /> {t("reports.view_raw_logs")}</span>
                </button>
            </div>
            {viewMode === "daily" && !isLoading && reportData.length > 0 && (
                <>
                    <div className="flex items-center gap-1.5 bg-slate-900 dark:bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-800 dark:border-slate-700 shadow-sm">
                        <FileBarChart className="text-primary-400" size={12} />
                        <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest">{t("reports.quick_stats.std_hours")}</span>
                        <span className="font-mono font-bold text-white text-xs tracking-tighter">{totalStdHours}h</span>
                    </div>
                    {Number(totalOtHours) > 0 && (
                        <div className="flex items-center gap-1.5 bg-gradient-to-r from-orange-900 to-orange-950 px-3 py-1.5 rounded-lg border border-orange-800 shadow-sm">
                            <Clock className="text-orange-400" size={12} />
                            <span className="text-[9px] font-bold text-orange-400/70 uppercase tracking-widest">{t("reports.quick_stats.ot_hours")}</span>
                            <span className="font-mono font-bold text-orange-100 text-xs tracking-tighter">{totalOtHours}h</span>
                        </div>
                    )}
                </>
            )}
        </>
    );
}

function EmployeeDailyDrawerModals({
    confirmDialog,
    reprocessMutationPending,
    setConfirmDialog,
    handleConfirmReprocess,
    confirmModalData,
    confirmMutationPending,
    setConfirmModalData,
    handleConfirmSubmit,
    resolveModalData,
    resolveUnscheduledMutationPending,
    setResolveModalData,
    handleResolveSubmit,
    shifts
}: any) {
    return (
        <>
            <ConfirmReprocessModal
                dialogData={confirmDialog}
                isOpen={!!confirmDialog}
                isPending={reprocessMutationPending}
                key={confirmDialog ? `reprocess-${confirmDialog.employeeId}-${confirmDialog.date}` : "reprocess-empty"}
                onClose={() => { setConfirmDialog(null); }}
                onConfirm={handleConfirmReprocess}
            />

            <ConfirmAttendanceModal
                existingRequest={confirmModalData?.existingRequest}
                isOpen={!!confirmModalData}
                isPending={confirmMutationPending}
                key={confirmModalData ? `confirm-${confirmModalData.row.date}` : "confirm-empty"}
                onClose={() => { setConfirmModalData(null); }}
                onSubmit={handleConfirmSubmit}
                row={confirmModalData?.row ?? null}
            />

            <ResolveUnscheduledModal
                isOpen={!!resolveModalData}
                isPending={resolveUnscheduledMutationPending}
                key={resolveModalData ? `resolve-${resolveModalData.row.date}` : "resolve-empty"}
                onClose={() => { setResolveModalData(null); }}
                onSubmit={handleResolveSubmit}
                row={resolveModalData?.row ?? null}
                shifts={shifts}
            />
        </>
    );
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
                const scrollContainer = contentRef.current?.querySelector(".overflow-auto") ?? contentRef.current;
                if (scrollContainer) {
                    scrollContainer.scrollBy({ top: e.key === "ArrowDown" ? 100 : -100, behavior: "smooth" });
                }
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => { window.removeEventListener("keydown", handleKeyDown); };
    }, [isOpen, viewMode, setViewMode]);

    return (
        <>
            <Sheet
                className="w-[95vw] xl:w-[1200px] !max-w-[95vw]"
                contentClassName="!overflow-hidden"
                headerExtra={
                    <EmployeeDailyDrawerHeader
                        isLoading={isLoading}
                        reportData={reportData}
                        setViewMode={setViewMode}
                        t={t}
                        totalOtHours={totalOtHours}
                        totalStdHours={totalStdHours}
                        viewMode={viewMode}
                    />
                }
                isOpen={isOpen}
                onClose={onClose}
                subtitle={`${formatDate(dateRange.start)} — ${formatDate(dateRange.end)}`}
                title={employeeName}
            >
                <div className="-mx-10 -mt-4 -mb-10 h-[calc(100%+theme(spacing.14))] flex flex-col" ref={contentRef}>
                    {viewMode === "raw" ? (
                        <RawLogsTable
                            fetchNextPage={fetchNextPage}
                            hasNextPage={hasNextPage}
                            isFetchingNextPage={isFetchingNextPage}
                            rawLogsData={rawLogsData}
                            rawLogsLoading={rawLogsLoading}
                        />
                    ) : (
                        <DailyReportTable
                            adjustmentMap={adjustmentMap}
                            confirmingRow={confirmingRow}
                            data={reportData}
                            expandedDebugRows={expandedDebugRows}
                            formatDate={formatDate}
                            formatTime={formatTime}
                            hideEmployeeName
                            isDebugMode={isDebugMode}
                            isLoading={isLoading}
                            onApproveOt={handleApproveOt}
                            onConfirmClick={handleConfirmClick}
                            onReprocessClick={handleReprocessClick}
                            onResolveUnscheduled={handleResolveUnscheduled}
                            onToggleExpandRow={toggleExpandRow}
                            reprocessingRow={reprocessingRow}
                            t={t}
                            workUnitMinutes={workUnitMinutes}
                        />
                    )}
                </div>
            </Sheet>

            <EmployeeDailyDrawerModals
                confirmDialog={confirmDialog}
                confirmModalData={confirmModalData}
                confirmMutationPending={confirmMutationPending}
                handleConfirmReprocess={handleConfirmReprocess}
                handleConfirmSubmit={handleConfirmSubmit}
                handleResolveSubmit={handleResolveSubmit}
                reprocessMutationPending={reprocessMutationPending}
                resolveModalData={resolveModalData}
                resolveUnscheduledMutationPending={resolveUnscheduledMutationPending}
                setConfirmDialog={setConfirmDialog}
                setConfirmModalData={setConfirmModalData}
                setResolveModalData={setResolveModalData}
                shifts={shifts}
            />
        </>
    );
}
