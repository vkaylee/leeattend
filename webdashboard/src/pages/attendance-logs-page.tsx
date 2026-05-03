import type { TFunction } from "i18next";

import { useQuery, useInfiniteQuery, useQueryClient, useMutation } from "@tanstack/react-query";
import { useNavigate, Link } from "@tanstack/react-router";
import {
  Download,
  Calendar,
  Upload,
  Clock,

  Users,
  CalendarRange,
  AlertTriangle,
  BarChart3 
} from "lucide-react";
import React, { useState, useMemo, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { format, endOfMonth } from "date-fns";

import { EmployeeSearch } from "@/components/common/employee-search";
import FloatingRecordBadge from "@/components/common/floating-record-badge";
import { MonthYearPicker } from "@/components/common/month-year-picker";
import PermissionGate from "@/components/common/permission-gate";
import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import { RecordedLogsTable } from "@/components/features/attendance/recorded-logs-table";
import { StagingLogsTable } from "@/components/features/attendance/staging-logs-table";
import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { fetchRawAttendanceLogs, getCurrentWorkspaceId, fetchDevices, fetchStagingLogs, resolveStagingLog, dismissStagingLog } from "@/lib/api";
import { RawAttendanceLog, Employee } from "@/lib/api/types";
import { cn } from "@/lib/utils";

type TabType = "recorded" | "unknown";

interface AttendanceLogsHeaderProps {
  dateRange: { start: string; end: string };
  setIsImportDialogOpen: (open: boolean) => void;
  t: TFunction;
}

function AttendanceLogsHeader({ dateRange, setIsImportDialogOpen, t }: AttendanceLogsHeaderProps) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200" data-testid="attendance-logs-title">{t("attendance_logs.title")}</h1>
        <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("attendance_logs.subtitle")}</p>
      </div>
      <div className="flex items-center gap-3">
        <Link
          className="flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-5 py-2.5 rounded-2xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-[10px] uppercase tracking-widest shadow-sm"
          search={{ from: dateRange.start, to: dateRange.end } as Record<string, unknown>}
          to="/reports"
        >
          <BarChart3 size={14} /> {t("attendance_logs.view_reports")}
        </Link>
        <Button icon={<Download size={14} />} onClick={() => { window.print(); }} size="sm" variant="outline">{t("attendance_logs.export_btn")}</Button>
        <PermissionGate permission="attendance_staging:manage">
          <Button icon={<Upload size={14} />} onClick={() => { setIsImportDialogOpen(true); }} size="sm">{t("attendance_logs.import_btn")}</Button>
        </PermissionGate>
      </div>
    </div>
  );
}

interface AttendanceLogsTabsProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  stagingCount?: number;
  t: TFunction;
}

function AttendanceLogsTabs({ activeTab, setActiveTab, stagingCount, t }: AttendanceLogsTabsProps) {
  return (
    <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm p-1 flex items-center gap-1 w-fit">
      <button
        className={cn(
          "px-6 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all flex items-center gap-2",
          activeTab === "recorded"
            ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800"
        )}
        onClick={() => { setActiveTab("recorded"); }}
      >
        <Clock size={14} />
        {t("attendance_logs.tabs.recorded")}
      </button>
      <button
        className={cn(
          "px-6 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all flex items-center gap-2",
          activeTab === "unknown"
            ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 shadow-sm border border-amber-100 dark:border-amber-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800"
        )}
        onClick={() => { setActiveTab("unknown"); }}
      >
        <AlertTriangle size={14} />
        {t("attendance_logs.tabs.unknown")}
        {(stagingCount ?? 0) > 0 && (
          <span className="ml-1 px-2 py-0.5 bg-amber-100 dark:bg-amber-950 text-amber-700 dark:text-amber-200 rounded-full text-[9px] font-bold">
            {stagingCount}
          </span>
        )}
      </button>
    </div>
  );
}

interface DateRange {
  end: string;
  start: string;
}

interface AttendanceLogsFiltersProps {
  dateRange: DateRange;
  employeeId: string;
  filterMode: "this_month" | "last_month" | "custom" | "specific_month";
  handleFilterModeChange: (mode: "this_month" | "last_month" | "custom" | "specific_month") => void;
  handleSpecificMonthChange: (month: number, year: number) => void;
  setDateRange: (range: DateRange | ((prev: DateRange) => DateRange)) => void;
  setEmployeeId: (id: string) => void;
  specificMonth: { month: number; year: number };
  t: TFunction;
  wsId: string | null;
}

function AttendanceLogsFilters({ filterMode, handleFilterModeChange, specificMonth, handleSpecificMonthChange, employeeId, setEmployeeId, wsId, t, dateRange, setDateRange }: AttendanceLogsFiltersProps) {
  return (
    <>
      <div className="bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm p-2 flex flex-col xl:flex-row xl:items-center justify-between gap-4">
        <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-2xl p-1 shadow-inner border border-slate-200 dark:border-slate-700/50">
          {(["this_month", "last_month"] as const).map(mode => (
            <button className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === mode ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} key={mode} onClick={() => { handleFilterModeChange(mode); }}>{t(`common.filters.${mode}`)}</button>
          ))}
          <div className="h-6 w-px bg-slate-200 dark:bg-slate-600 mx-1" />
          <div className="relative group">
            <button className={cn("flex items-center gap-2 px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "specific_month" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}>
              <CalendarRange size={14} /> {filterMode === "specific_month" ? `T${specificMonth.month}/${specificMonth.year}` : t("common.filters.specific_month")}
            </button>
            <div className="absolute left-0 top-full mt-2 z-[100] hidden group-hover:block animate-in zoom-in-95 duration-200 origin-top-left"><MonthYearPicker month={specificMonth.month} onChange={handleSpecificMonthChange} year={specificMonth.year} /></div>
          </div>
          <button className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "custom" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} onClick={() => { handleFilterModeChange("custom"); }}>{t("common.filters.custom")}</button>
        </div>

        <div className="flex-1 max-w-md flex items-center gap-3 px-2">
          <div className="flex-1"><EmployeeSearch clearOnEscape data-testid="employee-search" keyboardShortcut={["/", "Enter"]} onChange={(value) => { setEmployeeId((Array.isArray(value) ? value[0] : value) || "all"); }} placeholder={employeeId === "all" ? t("common.all_employees") : undefined} value={employeeId === "all" ? "" : employeeId} workspaceId={wsId ?? ""} /></div>
          {employeeId !== "all" && <button className="p-2.5 bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl transition-all" onClick={() => { setEmployeeId("all"); }}><Users size={18} /></button>}
        </div>
      </div>

      {filterMode === "custom" && (
        <div className="bg-white dark:bg-slate-900 p-6 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-top-2 duration-300 max-w-2xl">
          <div className="space-y-2">
            <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.from_date")}</label>
            <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" onChange={(e) => { setDateRange((previous) => ({ ...previous, start: e.target.value })); }} type="date" value={dateRange.start} />
          </div>
          <div className="space-y-2">
            <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.to_date")}</label>
            <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" onChange={(e) => { setDateRange((previous) => ({ ...previous, end: e.target.value })); }} type="date" value={dateRange.end} />
          </div>
        </div>
      )}
    </>
  );
}


export default function AttendanceLogsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  // URL search params — read once on mount for cross-page navigation
  const searchParams = new URLSearchParams(window.location.search);
  const setSearchParams = (_params: Record<string, string>, _options?: unknown) => {
    // No-op: we use local state, not URL for this page
  };
  const wsId = getCurrentWorkspaceId();
  const { formatDate, formatTime } = useDateTimeFormatter();
 
  const queryClient = useQueryClient();

  // Initialize from URL search params if present
  const urlFrom = searchParams.get("from");
  const urlTo = searchParams.get("to");
  const urlEmployee = searchParams.get("employee");

  const [activeTab, setActiveTab] = useState<TabType>("recorded");
  const PAGE_SIZE = 20;
  const [employeeId, setEmployeeId] = useState<string>(urlEmployee ?? "all");

  const getMonthRange = (offset = 0) => {
    const d = new Date(); d.setMonth(d.getMonth() + offset);
    const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), 'yyyy-MM-dd');
    const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), 'yyyy-MM-dd');
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<"this_month" | "last_month" | "custom" | "specific_month">(
    urlFrom && urlTo ? "custom" : "this_month"
  );
  const [dateRange, setDateRange] = useState(
    urlFrom && urlTo ? { start: urlFrom, end: urlTo } : getMonthRange(0)
  );
  const [specificMonth, setSpecificMonth] = useState({ month: new Date().getMonth() + 1, year: new Date().getFullYear() });
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  // Clear URL params after initialization
  useEffect(() => {
    if (urlFrom || urlTo || urlEmployee) {
      setSearchParams({}, { replace: true });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- Intentional: effect should only fire when listed deps change
  }, []);

  const handleFilterModeChange = useCallback((mode: "this_month" | "last_month" | "custom" | "specific_month") => {
    setFilterMode(mode);
    if (mode === "this_month") setDateRange(getMonthRange(0));
    else if (mode === "last_month") setDateRange(getMonthRange(-1));
  }, []);

  const handleSpecificMonthChange = useCallback((m: number, y: number) => {
    setSpecificMonth({ month: m, year: y }); setFilterMode("specific_month");
    const start = format(new Date(y, m - 1, 1), 'yyyy-MM-dd');
    const end = format(endOfMonth(new Date(y, m - 1, 1)), 'yyyy-MM-dd');
    setDateRange({ start, end });
  }, []);

  const { employees: _employees } = useAllEmployees(wsId);

  // Optimized lookup map to prevent O(N) search in render loop
  const employeeMap = useMemo(() => {
    const map = new Map<string, Employee>();
    _employees.forEach(emp => {
      map.set(emp.id, emp);
    });
    return map;
  }, [_employees]);

  const {
    data: recordedPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["attendance-raw-logs", wsId, employeeId, dateRange],
    queryFn: ({ pageParam }) => {
      const from = dateRange.start ? new Date(`${dateRange.start}T00:00:00`).toISOString() : undefined;
      const to = dateRange.end ? new Date(`${dateRange.end}T23:59:59`).toISOString() : undefined;
      return fetchRawAttendanceLogs({ employee_id: employeeId === "all" ? undefined : employeeId, from, to, page: pageParam, limit: PAGE_SIZE });
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab === "recorded",
  });

  const allRecordedLogs = useMemo(() => {
    if (!recordedPages) return [];
    return recordedPages.pages.flatMap((p) => p.data);
  }, [recordedPages]);

  const recordedTotal = recordedPages?.pages[0]?.total ?? 0;

  const {
    data: stagingPages,
    isLoading: stagingLoading,
    fetchNextPage: fetchNextStagingPage,
    hasNextPage: hasNextStagingPage,
    isFetchingNextPage: isFetchingNextStagingPage,
  } = useInfiniteQuery({
    queryKey: ["staging-logs", wsId],
    queryFn: ({ pageParam }) => fetchStagingLogs(wsId!, pageParam, PAGE_SIZE),
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab === "unknown",
  });

  const allStagingLogs = useMemo(() => {
    if (!stagingPages) return [];
    return stagingPages.pages.flatMap((p) => p.data);
  }, [stagingPages]);

  const stagingTotal = stagingPages?.pages[0]?.total ?? 0;

  const { data: stagingCount } = useQuery({
    queryKey: ["staging-logs-count", wsId],
    queryFn: async () => {
      const result = await fetchStagingLogs(wsId!, 1, 1);
      return result.total;
    },
    enabled: !!wsId,
  });

  const { data: devicesData } = useQuery({
    queryKey: ["devices", wsId],
    queryFn: () => fetchDevices(),
    enabled: !!wsId,
  });

  const dismissMutation = useMutation({
    mutationFn: (logId: string) => dismissStagingLog(wsId!, logId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      void queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
    },
  });

  const resolveMutation = useMutation({
    mutationFn: ({ logId, employeeId }: { logId: string; employeeId: string }) =>
      resolveStagingLog(wsId!, logId, employeeId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      void queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-raw-logs"] });
    },
  });

  const getEmployeeName = (log: RawAttendanceLog) => {
    if (log.employee_full_name) return log.employee_full_name;
    const emp = employeeMap.get(log.employee_id);
    return emp ? emp.full_name : t("common.system");
  };
  const getEmployeeID = (log: RawAttendanceLog) => {
    if (log.employee_id_number) return log.employee_id_number;
    const emp = employeeMap.get(log.employee_id);
    return emp ? emp.employee_id_number : "";
  };

  // O(1) device lookup map (same pattern as employeeMap)
  const deviceMap = useMemo(() => {
    const map = new Map<string, string>();
    if (devicesData) {
      devicesData.forEach(d => map.set(d.id, d.name));
    }
    return map;
  }, [devicesData]);

  const getDeviceName = useCallback((deviceId?: string | null): string => {
    if (!deviceId) return "";
    return deviceMap.get(deviceId) ?? "";
  }, [deviceMap]);

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  });

  const loadMoreStagingRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage: hasNextStagingPage,
    isFetchingNextPage: isFetchingNextStagingPage,
    fetchNextPage: fetchNextStagingPage,
  });

  const handleCreateEmployee = (enrollmentId: string) => {
    void navigate({ to: "/employees", search: { enrollment_id: enrollmentId } });
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700 pb-20">
      <AttendanceLogsHeader
        dateRange={dateRange}
        setIsImportDialogOpen={setIsImportDialogOpen}
        t={t}
      />

      <AttendanceLogsTabs
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        stagingCount={stagingCount}
        t={t}
      />

      {activeTab === "recorded" && (
        <>
          <AttendanceLogsFilters
            dateRange={dateRange}
            employeeId={employeeId}
            filterMode={filterMode}
            handleFilterModeChange={handleFilterModeChange}
            handleSpecificMonthChange={handleSpecificMonthChange}
            setDateRange={setDateRange}
            setEmployeeId={setEmployeeId}
            specificMonth={specificMonth}
            t={t}
            wsId={wsId}
          />
          
          <RecordedLogsTable
            formatDate={formatDate}
            formatTime={formatTime}
            getDeviceName={getDeviceName}
            getEmployeeID={getEmployeeID}
            getEmployeeName={getEmployeeName}
            isFetchingNextPage={isFetchingNextPage}
            isLoading={isLoading}
            loadMoreRef={loadMoreRef}
            logs={allRecordedLogs}
          />
        </>
      )}

      {activeTab === "unknown" && (
        <StagingLogsTable
          formatDate={formatDate}
          formatTime={formatTime}
          getDeviceName={getDeviceName}
          isDismissing={dismissMutation.isPending}
          isFetchingNextPage={isFetchingNextStagingPage}
          isLoading={stagingLoading}
          loadMoreRef={loadMoreStagingRef}
          logs={allStagingLogs}
          onCreateEmployee={handleCreateEmployee}
          onDismiss={(logId) => { dismissMutation.mutate(logId); }}
          onResolve={(logId, employeeId) => { resolveMutation.mutate({ logId, employeeId }); }}
          wsId={wsId!}
        />
      )}

      {/* Floating record count badge */}
      {activeTab === "recorded" ? (
        <FloatingRecordBadge label={t("common.pagination.records")} loaded={allRecordedLogs.length} total={recordedTotal} />
      ) : (
        <FloatingRecordBadge label={t("common.pagination.records")} loaded={allStagingLogs.length} total={stagingTotal} variant="amber" />
      )}

      <AttendanceImportDialog endpoint="/attendance/logs/upload" isOpen={isImportDialogOpen} onClose={() => { setIsImportDialogOpen(false); }} targetType="log" title={t("attendance_logs.modal.import_title")} />
    </div>
  );
}
