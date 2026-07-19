import type { TFunction } from "i18next";

import { CalendarCheck, FileBarChart, Clock, Briefcase, ChevronDown } from "lucide-react";
import React, { useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { DailyAttendanceReport, AttendanceAdjustmentRequest, Shift } from "@/lib/api";

import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
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
    employmentId?: string;
    employments?: { employmentId: string; employerName: string; employeeIdNumber?: string }[];
    isOpen: boolean;
    onClose: () => void;
    workspaceId: string;
}

interface EmployeeDailyDrawerHeaderProps {
    employments?: { employmentId: string; employerName: string; employeeIdNumber?: string }[];
    isLoading: boolean;
    onEmploymentChange: (id: string) => void;
    reportData: DailyAttendanceReport[];
    selectedEmploymentId?: string;
    setViewMode: (mode: "daily" | "raw") => void;
    t: TFunction;
    totalOtHours: string | number;
    totalStdHours: string | number;
    viewMode: "daily" | "raw";
}

  // eslint-disable-next-line complexity
function EmployeeDailyDrawerHeader({
    viewMode,
    setViewMode,
    isLoading,
    reportData,
    totalStdHours,
    totalOtHours,
    t,
    employments = [],
    selectedEmploymentId = "",
    onEmploymentChange,
}: EmployeeDailyDrawerHeaderProps) {
    return (
        <>
            {employments.length > 1 && (
                <div className="mr-4">
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button
                                className="flex items-center gap-1.5 px-4 py-2 border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 rounded-lg text-xs font-bold text-slate-700 dark:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-700/50 transition-all h-auto"
                                variant="ghost"
                            >
                                <Briefcase className="text-primary-500" size={12} />
                                <span>
                                    {employments.find(e => e.employmentId === selectedEmploymentId)?.employerName ?? t("common.employer")}
                                </span>
                                <ChevronDown className="opacity-50" size={12} />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent
                            align="start"
                            className="w-64 p-2 rounded-2xl shadow-premium border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/95 backdrop-blur-xl"
                            sideOffset={8}
                        >
                            {employments.map((emp) => (
                                <Button
                                    className={`w-full justify-start text-left px-3 py-2 rounded-xl text-xs font-bold transition-all h-auto mb-1 ${
                                        selectedEmploymentId === emp.employmentId
                                            ? "bg-primary-50 dark:bg-primary-950/30 text-primary-600 dark:text-primary-400"
                                            : "text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800/50"
                                    }`}
                                    key={emp.employmentId}
                                    onClick={() => { onEmploymentChange(emp.employmentId); }}
                                    variant="ghost"
                                >
                                    <div className="flex flex-col">
                                        <span>{emp.employerName}</span>
                                        {!!emp.employeeIdNumber && <span className="text-[9px] text-slate-400 font-mono mt-0.5">{emp.employeeIdNumber}</span>}
                                    </div>
                                </Button>
                            ))}
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            )}
            <div className="flex bg-slate-100 dark:bg-slate-800 rounded-lg p-1 border border-slate-200 dark:border-slate-700">
                <Button
                    className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all h-auto ${viewMode === "daily"
                        ? "bg-white dark:bg-slate-700 text-primary-600 dark:text-primary-400 shadow-sm hover:bg-white dark:hover:bg-slate-700"
                        : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
                        }`}
                    onClick={() => { setViewMode("daily"); }}
                    variant="ghost"
                >
                    {t("reports.tabs.daily")}
                </Button>
                <Button
                    className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all h-auto ${viewMode === "raw"
                        ? "bg-white dark:bg-slate-700 text-primary-600 dark:text-primary-400 shadow-sm hover:bg-white dark:hover:bg-slate-700"
                        : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
                        }`}
                    onClick={() => { setViewMode("raw"); }}
                    variant="ghost"
                >
                    <span className="flex items-center gap-1.5"><CalendarCheck size={12} /> {t("reports.view_raw_logs")}</span>
                </Button>
            </div>
            {viewMode === "daily" && !isLoading && reportData.length > 0 ? (
                <>
                    <div className="flex items-center gap-1.5 bg-slate-900 dark:bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-800 dark:border-slate-700 shadow-sm">
                        <FileBarChart className="text-primary-400" size={12} />
                        <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest">{t("reports.quick_stats.std_hours")}</span>
                        <span className="font-mono font-bold text-white text-xs tracking-tighter">{totalStdHours}h</span>
                    </div>
                    {Number(totalOtHours) > 0 ? (
                        <div className="flex items-center gap-1.5 bg-gradient-to-r from-orange-900 to-orange-950 px-3 py-1.5 rounded-lg border border-orange-800 shadow-sm">
                            <Clock className="text-orange-400" size={12} />
                            <span className="text-[9px] font-bold text-orange-400/70 uppercase tracking-widest">{t("reports.quick_stats.ot_hours")}</span>
                            <span className="font-mono font-bold text-orange-100 text-xs tracking-tighter">{totalOtHours}h</span>
                        </div>
                    ) : null}
                </>
            ) : null}
        </>
    );
}

interface EmployeeDailyDrawerModalsProps {
    confirmDialog: { show?: boolean; employeeId: string; employeeName: string; date: string } | null;
    confirmModalData: { row: DailyAttendanceReport; existingRequest?: AttendanceAdjustmentRequest } | null;
    confirmMutationPending: boolean;
    handleConfirmReprocess: () => void;
    handleConfirmSubmit: (data: { punchIn: string; punchOut: string; reason: string }) => void;
    handleResolveSubmit: (shiftId: string) => void;
    reprocessMutationPending: boolean;
    resolveModalData: { row: DailyAttendanceReport } | null;
    resolveUnscheduledMutationPending: boolean;
    setConfirmDialog: (data: { show: boolean; employeeId: string; employeeName: string; date: string } | null) => void;
    setConfirmModalData: (data: { row: DailyAttendanceReport; existingRequest?: AttendanceAdjustmentRequest } | null) => void;
    setResolveModalData: (data: { row: DailyAttendanceReport } | null) => void;
    shifts?: Shift[];
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
}: EmployeeDailyDrawerModalsProps) {
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

function shouldIgnoreKeyboardEvent(e: KeyboardEvent) {
    return e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
}

function handleTabSwitch(e: KeyboardEvent, viewMode: "daily" | "raw", setViewMode: (mode: "daily" | "raw") => void) {
    if (e.key === "ArrowRight" && viewMode === "daily") setViewMode("raw");
    if (e.key === "ArrowLeft" && viewMode === "raw") setViewMode("daily");
}

function handleScroll(e: KeyboardEvent, contentRef: React.RefObject<HTMLDivElement | null>) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const scrollContainer = contentRef.current?.querySelector(".overflow-auto") ?? contentRef.current;
    if (scrollContainer) {
        scrollContainer.scrollBy({ top: e.key === "ArrowDown" ? 100 : -100, behavior: "smooth" });
    }
}

function useDrawerKeyboardNavigation(isOpen: boolean, viewMode: "daily" | "raw", setViewMode: (mode: "daily" | "raw") => void, contentRef: React.RefObject<HTMLDivElement | null>) {
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (shouldIgnoreKeyboardEvent(e)) return;
            handleTabSwitch(e, viewMode, setViewMode);
            handleScroll(e, contentRef);
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => { window.removeEventListener("keydown", handleKeyDown); };
    }, [isOpen, viewMode, setViewMode, contentRef]);
}

export default function EmployeeDailyDrawer(props: EmployeeDailyDrawerProps) {
    const { isOpen, onClose, employeeName, dateRange } = props;
    const { t } = useTranslation();
    const { formatDate, formatTime } = useDateTimeFormatter();
    const { isEnabled: isDebugMode } = useDebug();
    const contentRef = useRef<HTMLDivElement>(null);

    const data = useEmployeeDailyDrawer(props);

    // Keyboard navigation for tabs + scroll
    useDrawerKeyboardNavigation(isOpen, data.viewMode, data.setViewMode, contentRef);

    return (
        <>
            <Sheet
                className="w-[95vw] xl:w-[1200px] !max-w-[95vw]"
                contentClassName="!overflow-hidden"
                headerExtra={
                    <EmployeeDailyDrawerHeader
                        employments={props.employments}
                        isLoading={data.isLoading}
                        onEmploymentChange={data.setSelectedEmploymentId}
                        reportData={data.reportData}
                        selectedEmploymentId={data.selectedEmploymentId}
                        setViewMode={data.setViewMode}
                        t={t}
                        totalOtHours={data.totalOtHours}
                        totalStdHours={data.totalStdHours}
                        viewMode={data.viewMode}
                    />
                }
                isOpen={isOpen}
                onClose={onClose}
                subtitle={`${formatDate(dateRange.start)} — ${formatDate(dateRange.end)}`}
                title={employeeName}
            >
                <div className="-mx-10 -mt-4 -mb-10 h-[calc(100%+theme(spacing.14))] flex flex-col" ref={contentRef}>
                    {data.viewMode === "raw" ? (
                        <RawLogsTable
                            fetchNextPage={data.fetchNextPage}
                            hasNextPage={data.hasNextPage}
                            isFetchingNextPage={data.isFetchingNextPage}
                            rawLogsData={data.rawLogsData}
                            rawLogsLoading={data.rawLogsLoading}
                        />
                    ) : (
                        <DailyReportTable
                            adjustmentMap={data.adjustmentMap}
                            confirmingRow={data.confirmingRow}
                            data={data.reportData}
                            expandedDebugRows={data.expandedDebugRows}
                            formatDate={formatDate}
                            formatTime={formatTime}
                            hideEmployeeName
                            isDebugMode={isDebugMode}
                            isLoading={data.isLoading}
                            onApproveOt={data.handleApproveOt}
                            onConfirmClick={data.handleConfirmClick}
                            onReprocessClick={data.handleReprocessClick}
                            onResolveUnscheduled={data.handleResolveUnscheduled}
                            onToggleExpandRow={data.toggleExpandRow}
                            reprocessingRow={data.reprocessingRow}
                            t={t}
                        />
                    )}
                </div>
            </Sheet>

            <EmployeeDailyDrawerModals
                confirmDialog={data.confirmDialog}
                confirmModalData={data.confirmModalData}
                confirmMutationPending={data.confirmMutationPending}
                handleConfirmReprocess={data.handleConfirmReprocess}
                handleConfirmSubmit={data.handleConfirmSubmit}
                handleResolveSubmit={data.handleResolveSubmit}
                reprocessMutationPending={data.reprocessMutationPending}
                resolveModalData={data.resolveModalData}
                resolveUnscheduledMutationPending={data.resolveUnscheduledMutationPending}
                setConfirmDialog={data.setConfirmDialog}
  // eslint-disable-next-line max-lines
                setConfirmModalData={data.setConfirmModalData}
                setResolveModalData={data.setResolveModalData}
                shifts={data.shifts}
            />
        </>
    );
}
