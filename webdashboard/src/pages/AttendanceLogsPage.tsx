import { useState, useMemo, useEffect, useCallback } from "react";
import React from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useInfiniteQuery, useQueryClient, useMutation } from "@tanstack/react-query";
import { fetchRawAttendanceLogs, fetchEmployees, getCurrentWorkspaceId, fetchDevices, fetchStagingLogs, resolveStagingLog, dismissStagingLog } from "@/lib/api";
import { RawAttendanceLog, Employee } from "@/lib/api/types";
import { format, endOfMonth } from "date-fns";
import FloatingRecordBadge from "@/components/common/FloatingRecordBadge";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import InfiniteScrollSentinel from "@/components/common/InfiniteScrollSentinel";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import {
  Download,
  Calendar,
  Upload,
  Clock,

  Users,
  CalendarRange,
  AlertTriangle,
  UserPlus,
  Trash2,
  Link2,
  Check,
  Loader2
} from "lucide-react";
import { cn } from "@/lib/utils";
import Button from "@/components/ui/Button";
import StatusBadge from "@/components/ui/StatusBadge";
import { LABEL_BASE } from "@/components/ui/FormField";
import AttendanceImportDialog from "@/components/features/attendance/AttendanceImportDialog";
import { RecordedLogsTable } from "@/components/features/attendance/RecordedLogsTable";
import { StagingLogsTable } from "@/components/features/attendance/StagingLogsTable";
import PermissionGate from "@/components/common/PermissionGate";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import { useNavigate, Link, useSearch } from "@tanstack/react-router";
import { BarChart3 } from "lucide-react";
import { TableSkeleton } from "@/components/common/TableSkeleton";

type TabType = "recorded" | "unknown";

export default function AttendanceLogsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  // URL search params — read once on mount for cross-page navigation
  const searchParams = new URLSearchParams(window.location.search);
  const setSearchParams = (_params: Record<string, string>, _opts?: any) => {
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
  const [employeeId, setEmployeeId] = useState<string>(urlEmployee || "all");

  const getMonthRange = (offset: number = 0) => {
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
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

  const { data: employeesData } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId!, 1, 3000), enabled: !!wsId });
  const employees = employeesData?.data || [];

  // Optimized lookup map to prevent O(N) search in render loop
  const employeeMap = useMemo(() => {
    const map = new Map<string, Employee>();
    const emps = employeesData?.data || [];
    emps.forEach(emp => {
      map.set(emp.id, emp);
    });
    return map;
  }, [employeesData?.data]);

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
      return fetchRawAttendanceLogs({ employee_id: employeeId === "all" ? undefined : employeeId, from, to, page: pageParam as number, limit: PAGE_SIZE });
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data?.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab === "recorded",
  });

  const allRecordedLogs = useMemo(() => {
    if (!recordedPages) return [];
    return recordedPages.pages.flatMap((p) => p.data || []);
  }, [recordedPages]);

  const recordedTotal = recordedPages?.pages[0]?.total || 0;

  const {
    data: stagingPages,
    isLoading: stagingLoading,
    fetchNextPage: fetchNextStagingPage,
    hasNextPage: hasNextStagingPage,
    isFetchingNextPage: isFetchingNextStagingPage,
  } = useInfiniteQuery({
    queryKey: ["staging-logs", wsId],
    queryFn: ({ pageParam }) => fetchStagingLogs(wsId!, pageParam as number, PAGE_SIZE),
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data?.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab === "unknown",
  });

  const allStagingLogs = useMemo(() => {
    if (!stagingPages) return [];
    return stagingPages.pages.flatMap((p) => p.data || []);
  }, [stagingPages]);

  const stagingTotal = stagingPages?.pages[0]?.total || 0;

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
      queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
    },
  });

  const resolveMutation = useMutation({
    mutationFn: ({ logId, employeeId }: { logId: string; employeeId: string }) =>
      resolveStagingLog(wsId!, logId, employeeId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
      queryClient.invalidateQueries({ queryKey: ["attendance-raw-logs"] });
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
    navigate({ to: "/employees", search: { enrollment_id: enrollmentId } });
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div>
          <h1 data-testid="attendance-logs-title" className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200">{t("attendance_logs.title")}</h1>
          <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("attendance_logs.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <Link
            to="/reports"
            search={{ from: dateRange.start, to: dateRange.end } as any}
            className="flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-5 py-2.5 rounded-2xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-[10px] uppercase tracking-widest shadow-sm"
          >
            <BarChart3 size={14} /> {t("attendance_logs.view_reports")}
          </Link>
          <Button variant="outline" size="sm" onClick={() => window.print()} icon={<Download size={14} />}>{t("attendance_logs.export_btn")}</Button>
          <PermissionGate permission="attendance_staging:manage">
            <Button size="sm" onClick={() => setIsImportDialogOpen(true)} icon={<Upload size={14} />}>{t("attendance_logs.import_btn")}</Button>
          </PermissionGate>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm p-1 flex items-center gap-1 w-fit">
        <button
          onClick={() => { setActiveTab("recorded"); }}
          className={cn(
            "px-6 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all flex items-center gap-2",
            activeTab === "recorded"
              ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800"
          )}
        >
          <Clock size={14} />
          {t("attendance_logs.tabs.recorded", "Recorded Logs")}
        </button>
        <button
          onClick={() => { setActiveTab("unknown"); }}
          className={cn(
            "px-6 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all flex items-center gap-2",
            activeTab === "unknown"
              ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 shadow-sm border border-amber-100 dark:border-amber-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800"
          )}
        >
          <AlertTriangle size={14} />
          {t("attendance_logs.tabs.unknown", "Unidentified")}
          {(stagingCount ?? 0) > 0 && (
            <span className="ml-1 px-2 py-0.5 bg-amber-100 dark:bg-amber-950 text-amber-700 dark:text-amber-200 rounded-full text-[9px] font-bold">
              {stagingCount}
            </span>
          )}
        </button>
      </div>

      {activeTab === "recorded" && (
        <>
          <div className="bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm p-2 flex flex-col xl:flex-row xl:items-center justify-between gap-4">
            <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-2xl p-1 shadow-inner border border-slate-200 dark:border-slate-700/50">
              {["this_month", "last_month"].map(mode => (
                <button key={mode} onClick={() => handleFilterModeChange(mode as any)} className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === mode ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}>{t(`common.filters.${mode}`)}</button>
              ))}
              <div className="h-6 w-px bg-slate-200 dark:bg-slate-600 mx-1" />
              <div className="relative group">
                <button className={cn("flex items-center gap-2 px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "specific_month" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}>
                  <CalendarRange size={14} /> {filterMode === "specific_month" ? `T${specificMonth.month}/${specificMonth.year}` : t("common.filters.specific_month")}
                </button>
                <div className="absolute left-0 top-full mt-2 z-[100] hidden group-hover:block animate-in zoom-in-95 duration-200 origin-top-left"><MonthYearPicker month={specificMonth.month} year={specificMonth.year} onChange={handleSpecificMonthChange} /></div>
              </div>
              <button onClick={() => handleFilterModeChange("custom")} className={cn("px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all", filterMode === "custom" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}>{t("common.filters.custom")}</button>
            </div>

            <div className="flex-1 max-w-md flex items-center gap-3 px-2">
              <div className="flex-1"><EmployeeSearch data-testid="employee-search" workspaceId={wsId!} value={employeeId === "all" ? "" : employeeId} onChange={(val) => { setEmployeeId(val || "all"); }} placeholder={employeeId === "all" ? t("common.all_employees") : undefined} autoFocus keyboardShortcut={["/", "Enter"]} clearOnEscape /></div>
              {employeeId !== "all" && <button onClick={() => { setEmployeeId("all"); }} className="p-2.5 bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl transition-all"><Users size={18} /></button>}
            </div>
          </div>

          {filterMode === "custom" && (
            <div className="bg-white dark:bg-slate-900 p-6 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-top-2 duration-300 max-w-2xl">
              <div className="space-y-2">
                <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.from_date")}</label>
                <input type="date" value={dateRange.start} onChange={(e) => { setDateRange(prev => ({ ...prev, start: e.target.value })); }} className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" />
              </div>
              <div className="space-y-2">
                <label className={cn(LABEL_BASE, "flex items-center gap-2")}><Calendar size={12} /> {t("common.to_date")}</label>
                <input type="date" value={dateRange.end} onChange={(e) => { setDateRange(prev => ({ ...prev, end: e.target.value })); }} className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-xl px-4 py-3 outline-none focus:ring-4 focus:ring-primary-500/10 transition-all font-bold text-sm" />
              </div>
            </div>
          )}
          
          <RecordedLogsTable
            logs={allRecordedLogs}
            isLoading={isLoading}
            isFetchingNextPage={isFetchingNextPage}
            loadMoreRef={loadMoreRef}
            getEmployeeName={getEmployeeName}
            getEmployeeID={getEmployeeID}
            getDeviceName={getDeviceName}
            formatDate={formatDate}
            formatTime={formatTime}
          />
        </>
      )}

      {activeTab === "unknown" && (
        <StagingLogsTable
          logs={allStagingLogs}
          isLoading={stagingLoading}
          isFetchingNextPage={isFetchingNextStagingPage}
          loadMoreRef={loadMoreStagingRef}
          wsId={wsId!}
          getDeviceName={getDeviceName}
          formatDate={formatDate}
          formatTime={formatTime}
          onResolve={(logId, employeeId) => resolveMutation.mutate({ logId, employeeId })}
          onDismiss={(logId) => dismissMutation.mutate(logId)}
          isDismissing={dismissMutation.isPending}
          onCreateEmployee={handleCreateEmployee}
        />
      )}

      {/* Floating record count badge */}
      {activeTab === "recorded" ? (
        <FloatingRecordBadge loaded={allRecordedLogs.length} total={recordedTotal} label={t("common.pagination.records")} />
      ) : (
        <FloatingRecordBadge loaded={allStagingLogs.length} total={stagingTotal} label={t("common.pagination.records")} variant="amber" />
      )}

      <AttendanceImportDialog isOpen={isImportDialogOpen} onClose={() => setIsImportDialogOpen(false)} title={t("attendance_logs.modal.import_title")} endpoint="/attendance/logs/upload" targetType="log" />
    </div>
  );
}
