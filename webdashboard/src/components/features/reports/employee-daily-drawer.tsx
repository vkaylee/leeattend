import type { TFunction } from "i18next";

import { CalendarCheck, FileBarChart, Clock, Briefcase, ChevronDown } from "lucide-react";
import React, { useEffect, useId, useRef } from "react";
import { useTranslation } from "react-i18next";

import type { DailyAttendanceReport, AttendanceAdjustmentRequest, Shift } from "@/lib/api";

import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import Sheet from "@/components/ui/sheet";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { cn } from "@/lib/utils";

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
    employerId?: string;
    employers?: { employerId?: string; employerName: string; employeeIdNumber?: string }[];
    isOpen: boolean;
    onClose: () => void;
}

interface DrawerTabRefs {
    daily: React.RefObject<HTMLButtonElement | null>;
    raw: React.RefObject<HTMLButtonElement | null>;
}

interface EmployeeDailyDrawerHeaderProps {
    dailyTabRef: React.RefObject<HTMLButtonElement | null>;
    employers?: { employerId?: string; employerName: string; employeeIdNumber?: string }[];
    isLoading: boolean;
    onEmployerChange: (id: string | undefined) => void;
    rawTabRef: React.RefObject<HTMLButtonElement | null>;
    reportData: DailyAttendanceReport[];
    selectedEmployerId?: string;
    setViewMode: (mode: "daily" | "raw") => void;
    t: TFunction;
    tabIdPrefix: string;
    totalOtHours: string | number;
    totalStdHours: string | number;
    viewMode: "daily" | "raw";
}

interface DrawerMetricProps {
    icon: React.ReactNode;
    isLoading: boolean;
    label: string;
    shortLabel: string;
    tone: "neutral" | "overtime";
    value: string;
}

function DrawerMetric({ icon, isLoading, label, shortLabel, tone, value }: DrawerMetricProps) {
    const isOvertime = tone === "overtime";

    return (
        <div
            aria-busy={isLoading}
            className={cn(
                "flex min-h-11 min-w-0 items-center gap-2 rounded-lg border px-3 py-2 shadow-sm",
                isOvertime
                    ? "border-orange-800 bg-gradient-to-r from-orange-900 to-orange-950"
                    : "border-slate-800 bg-slate-900 dark:bg-slate-800",
            )}
        >
            {icon}
            <span className={cn(
                "min-w-0 truncate text-xs font-semibold tracking-normal",
                isOvertime ? "text-orange-200/90" : "text-slate-300",
            )}>
                <span className="sm:hidden">{shortLabel}</span>
                <span className="hidden sm:inline">{label}</span>
            </span>
            {isLoading ? (
                <span aria-hidden="true" className="ml-auto h-4 w-10 shrink-0 animate-pulse rounded bg-white/20" />
            ) : (
                <span className={cn(
                    "ml-auto shrink-0 font-mono text-sm font-bold tracking-tight",
                    isOvertime ? "text-orange-100" : "text-white",
                )}>
                    {value}
                </span>
            )}
        </div>
    );
}

function EmployeeDailyDrawerHeader({
    viewMode,
    setViewMode,
    isLoading,
    reportData,
    totalStdHours,
    totalOtHours,
    t,
    tabIdPrefix,
    dailyTabRef,
    rawTabRef,
    employers = [],
    selectedEmployerId = "",
    onEmployerChange,
}: EmployeeDailyDrawerHeaderProps) {
    const dailyTabId = `${tabIdPrefix}-daily-tab`;
    const rawTabId = `${tabIdPrefix}-raw-tab`;

    return (
        <div className="flex w-full min-w-0 flex-col gap-3 sm:w-auto sm:flex-row sm:flex-wrap sm:items-center sm:justify-end">
            {employers.length > 1 && (
                <div className="w-full min-w-0 sm:w-auto">
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button
                                className="flex min-h-11 w-full min-w-0 items-center justify-between gap-1.5 rounded-lg border border-slate-200 bg-white px-4 py-2 text-left text-xs font-bold text-slate-700 transition-all hover:bg-slate-50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700/50 sm:w-auto sm:max-w-64"
                                variant="ghost"
                            >
                                <span className="flex min-w-0 items-center gap-1.5">
                                    <Briefcase aria-hidden="true" className="shrink-0 text-primary-500" size={14} />
                                    <span className="truncate">{employers.find(e => e.employerId === selectedEmployerId)?.employerName ?? t("common.employer")}</span>
                                </span>
                                <ChevronDown aria-hidden="true" className="shrink-0 opacity-50" size={14} />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent
                            align="start"
                            className="w-[calc(100vw-2rem)] max-w-64 rounded-2xl border-slate-100 bg-white p-2 shadow-premium backdrop-blur-xl dark:border-slate-800 dark:bg-slate-900/95"
                            sideOffset={8}
                        >
                            {employers.map((employer) => (
                                <Button
                                    className={cn(
                                        "mb-1 min-h-11 w-full justify-start rounded-xl px-3 py-2 text-left text-xs font-bold transition-all",
                                        selectedEmployerId === employer.employerId
                                            ? "bg-primary-50 text-primary-600 dark:bg-primary-950/30 dark:text-primary-400"
                                            : "text-slate-600 hover:bg-slate-50 dark:text-slate-400 dark:hover:bg-slate-800/50",
                                    )}
                                    key={employer.employerId ?? "none"}
                                    onClick={() => { onEmployerChange(employer.employerId); }}
                                    variant="ghost"
                                >
                                    <div className="flex min-w-0 flex-col">
                                        <span className="truncate">{employer.employerName}</span>
                                        {!!employer.employeeIdNumber && <span className="mt-0.5 font-mono text-[10px] text-slate-400">{employer.employeeIdNumber}</span>}
                                    </div>
                                </Button>
                            ))}
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            )}
            <div aria-label={t("reports.tabs.label")} aria-orientation="horizontal" className="flex min-h-11 w-full min-w-0 rounded-lg border border-slate-200 bg-slate-100 p-1 dark:border-slate-700 dark:bg-slate-800 sm:w-auto" role="tablist">
                <Button
                    aria-controls={`${tabIdPrefix}-daily-panel`}
                    aria-selected={viewMode === "daily"}
                    className={cn(
                        "min-h-9 flex-1 rounded-md px-3 text-xs font-semibold tracking-normal transition-all sm:min-h-10 sm:flex-none",
                        viewMode === "daily"
                            ? "bg-white text-primary-600 shadow-sm hover:bg-white dark:bg-slate-700 dark:text-primary-400 dark:hover:bg-slate-700"
                            : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200",
                    )}
                    id={dailyTabId}
                    onClick={() => { setViewMode("daily"); }}
                    ref={dailyTabRef}
                    role="tab"
                    tabIndex={viewMode === "daily" ? 0 : -1}
                    type="button"
                    variant="ghost"
                >
                    <span className="sm:hidden">{t("reports.tabs.daily_short")}</span>
                    <span className="hidden sm:inline">{t("reports.tabs.daily")}</span>
                </Button>
                <Button
                    aria-controls={`${tabIdPrefix}-raw-panel`}
                    aria-selected={viewMode === "raw"}
                    className={cn(
                        "min-h-9 flex-1 rounded-md px-3 text-xs font-semibold tracking-normal transition-all sm:min-h-10 sm:flex-none",
                        viewMode === "raw"
                            ? "bg-white text-primary-600 shadow-sm hover:bg-white dark:bg-slate-700 dark:text-primary-400 dark:hover:bg-slate-700"
                            : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200",
                    )}
                    id={rawTabId}
                    onClick={() => { setViewMode("raw"); }}
                    ref={rawTabRef}
                    role="tab"
                    tabIndex={viewMode === "raw" ? 0 : -1}
                    type="button"
                    variant="ghost"
                >
                    <span className="flex items-center justify-center gap-1.5">
                        <CalendarCheck aria-hidden="true" size={14} />
                        <span className="sm:hidden">{t("reports.view_raw_logs_short")}</span>
                        <span className="hidden sm:inline">{t("reports.view_raw_logs")}</span>
                    </span>
                </Button>
            </div>
            <div className="grid w-full grid-cols-2 gap-2 sm:flex sm:w-auto">
                <DrawerMetric
                    icon={<FileBarChart aria-hidden="true" className="shrink-0 text-primary-400" size={14} />}
                    isLoading={isLoading}
                    label={t("reports.quick_stats.std_hours")}
                    shortLabel={t("reports.quick_stats.std_hours_short")}
                    tone="neutral"
                    value={`${reportData.length > 0 ? totalStdHours : "—"}h`}
                />
                <DrawerMetric
                    icon={<Clock aria-hidden="true" className="shrink-0 text-orange-400" size={14} />}
                    isLoading={isLoading}
                    label={t("reports.quick_stats.ot_hours")}
                    shortLabel={t("reports.quick_stats.ot_hours_short")}
                    tone="overtime"
                    value={`${reportData.length > 0 ? totalOtHours : "—"}h`}
                />
            </div>
        </div>
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

function handleTabSwitch(
    e: KeyboardEvent,
    viewMode: "daily" | "raw",
    setViewMode: (mode: "daily" | "raw") => void,
    tabRefs: DrawerTabRefs,
) {
    if (!(e.target instanceof Element) || !e.target.closest('[role="tablist"]')) return;

    const nextMode = e.key === "ArrowRight" || e.key === "End"
        ? "raw"
        : e.key === "ArrowLeft" || e.key === "Home"
            ? "daily"
            : null;

    if (!nextMode) return;

    e.preventDefault();
    tabRefs[nextMode].current?.focus();
    if (nextMode !== viewMode) setViewMode(nextMode);
}

function handleScroll(e: KeyboardEvent, contentRef: React.RefObject<HTMLDivElement | null>) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const scrollContainer = contentRef.current?.querySelector(".overflow-auto") ?? contentRef.current;
    if (scrollContainer) {
        scrollContainer.scrollBy({ top: e.key === "ArrowDown" ? 100 : -100, behavior: "smooth" });
    }
}

interface DrawerKeyboardNavigationOptions {
    contentRef: React.RefObject<HTMLDivElement | null>;
    isOpen: boolean;
    setViewMode: (mode: "daily" | "raw") => void;
    tabRefs: DrawerTabRefs;
    viewMode: "daily" | "raw";
}

function useDrawerKeyboardNavigation({
    contentRef,
    isOpen,
    setViewMode,
    tabRefs,
    viewMode,
}: DrawerKeyboardNavigationOptions) {
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (shouldIgnoreKeyboardEvent(e)) return;
            handleTabSwitch(e, viewMode, setViewMode, tabRefs);
            handleScroll(e, contentRef);
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => { window.removeEventListener("keydown", handleKeyDown); };
    }, [isOpen, viewMode, setViewMode, contentRef, tabRefs]);
}

export default function EmployeeDailyDrawer(props: EmployeeDailyDrawerProps) {
    const { isOpen, onClose, employeeName, dateRange } = props;
    const { t } = useTranslation();
    const { formatDate, formatTime } = useDateTimeFormatter();
    const contentRef = useRef<HTMLDivElement>(null);
    const tabIdPrefix = useId();
    const dailyTabRef = useRef<HTMLButtonElement>(null);
    const rawTabRef = useRef<HTMLButtonElement>(null);
    const data = useEmployeeDailyDrawer(props);

    // Keyboard navigation for tabs + scroll
    useDrawerKeyboardNavigation({
        contentRef,
        isOpen,
        setViewMode: data.setViewMode,
        tabRefs: { daily: dailyTabRef, raw: rawTabRef },
        viewMode: data.viewMode,
    });

    return (
        <>
            <Sheet
                className="!w-full !max-w-full sm:!w-[95vw] sm:!max-w-[95vw] xl:!w-[1200px]"
                contentClassName="!overflow-hidden"
                headerExtra={
                    <EmployeeDailyDrawerHeader
                        dailyTabRef={dailyTabRef}
                        employers={props.employers}
                        isLoading={data.isLoading}
                        onEmployerChange={data.setSelectedEmployerId}
                        rawTabRef={rawTabRef}
                        reportData={data.reportData}
                        selectedEmployerId={data.selectedEmployerId}
                        setViewMode={data.setViewMode}
                        t={t}
                        tabIdPrefix={tabIdPrefix}
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
                <div className="-mx-4 -mt-4 -mb-4 flex min-h-0 flex-1 flex-col sm:-mx-10 sm:-mb-10" ref={contentRef}>
                    <div
                        aria-labelledby={`${tabIdPrefix}-daily-tab`}
                        className="flex min-h-0 flex-1 flex-col"
                        hidden={data.viewMode !== "daily"}
                        id={`${tabIdPrefix}-daily-panel`}
                        role="tabpanel"
                        tabIndex={0}
                    >
                        {data.viewMode === "daily" ? (
                            <DailyReportTable
                                adjustmentMap={data.adjustmentMap}
                                confirmingRow={data.confirmingRow}
                                data={data.reportData}
                                expandedDebugRows={data.expandedDebugRows}
                                formatDate={formatDate}
                                formatTime={formatTime}
                                hideEmployeeName
                                isLoading={data.isLoading}
                                onApproveOt={data.handleApproveOt}
                                onConfirmClick={data.handleConfirmClick}
                                onReprocessClick={data.handleReprocessClick}
                                onResolveUnscheduled={data.handleResolveUnscheduled}
                                onToggleExpandRow={data.toggleExpandRow}
                                reprocessingRow={data.reprocessingRow}
                                t={(key, defaultValue) => t(key, { defaultValue })}
                            />
                        ) : null}
                    </div>
                    <div
                        aria-labelledby={`${tabIdPrefix}-raw-tab`}
                        className="flex min-h-0 flex-1 flex-col"
                        hidden={data.viewMode !== "raw"}
                        id={`${tabIdPrefix}-raw-panel`}
                        role="tabpanel"
                        tabIndex={0}
                    >
                        {data.viewMode === "raw" ? (
                            <RawLogsTable
                                fetchNextPage={data.fetchNextPage}
                                hasNextPage={data.hasNextPage}
                                isFetchingNextPage={data.isFetchingNextPage}
                                rawLogsData={data.rawLogsData}
                                rawLogsLoading={data.rawLogsLoading}
                            />
                        ) : null}
                    </div>
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
   
                setConfirmModalData={data.setConfirmModalData}
                setResolveModalData={data.setResolveModalData}
                shifts={data.shifts}
            />
        </>
    );
}
