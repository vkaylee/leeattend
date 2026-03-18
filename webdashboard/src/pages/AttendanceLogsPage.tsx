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
import PermissionGate from "@/components/common/PermissionGate";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import { useNavigate, useSearchParams, Link } from "react-router-dom";
import { BarChart3 } from "lucide-react";


type TabType = "recorded" | "unknown";

export default function AttendanceLogsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
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
  const [assigningLogId, setAssigningLogId] = useState<string | null>(null);

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
      setAssigningLogId(null);
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
    navigate(`/employees?enrollment_id=${encodeURIComponent(enrollmentId)}`);
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
            to={`/reports?from=${dateRange.start}&to=${dateRange.end}`}
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

          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead className="sticky top-0 z-10 bg-white/80 dark:bg-slate-900/80 backdrop-blur pb-2">
                  <tr className="bg-slate-50/80 dark:bg-slate-800/80 border-b border-slate-100 dark:border-slate-800 shadow-sm backdrop-blur">
                    <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.time")}</th>
                    <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
                    <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.source")}</th>
                    <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.status")}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                  {isLoading ? (
                    Array.from({ length: 5 }).map((_, i) => (
                      <tr key={`skeleton-${i}`} className="animate-pulse border-b border-slate-50 dark:border-slate-800/50">
                        <td className="px-8 py-5">
                          <div className="flex items-center gap-4">
                            <div className="w-10 h-10 rounded-xl bg-slate-200 dark:bg-slate-800"></div>
                            <div className="space-y-2">
                              <div className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded"></div>
                              <div className="h-3 w-16 bg-slate-200 dark:bg-slate-800 rounded"></div>
                            </div>
                          </div>
                        </td>
                        <td className="px-8 py-5">
                          <div className="space-y-2">
                            <div className="h-4 w-32 bg-slate-200 dark:bg-slate-800 rounded"></div>
                            <div className="h-3 w-20 bg-slate-200 dark:bg-slate-800 rounded"></div>
                          </div>
                        </td>
                        <td className="px-8 py-5">
                          <div className="h-6 w-20 bg-slate-200 dark:bg-slate-800 rounded-lg"></div>
                        </td>
                        <td className="px-8 py-5 text-right">
                          <div className="h-6 w-16 bg-slate-200 dark:bg-slate-800 rounded-full inline-block"></div>
                        </td>
                      </tr>
                    ))
                  ) : allRecordedLogs.length === 0 ? (
                    <tr>
                      <td colSpan={4} className="px-8 py-24 text-center">
                        <div className="flex flex-col items-center justify-center space-y-4">
                          <div className="w-16 h-16 bg-slate-50 dark:bg-slate-800/50 rounded-full flex items-center justify-center text-slate-300 dark:text-slate-600">
                            <CalendarRange size={32} />
                          </div>
                          <div>
                            <h3 className="text-sm font-bold text-slate-700 dark:text-slate-300 uppercase tracking-widest">{t("attendance_logs.messages.no_data", "No Attendance Logs Found")}</h3>
                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-2 max-w-sm mx-auto font-medium">Try adjusting your date range filter or selecting a different employee to find what you're looking for.</p>
                          </div>
                        </div>
                      </td>
                    </tr>
                  ) : (
                    <>
                      {allRecordedLogs.map((log: RawAttendanceLog) => (
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
                            <div className="font-bold text-slate-800 dark:text-slate-100 text-sm tracking-tight uppercase">{getEmployeeName(log)}</div>
                            <div className="text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest mt-0.5">{getEmployeeID(log) || "SYSTEM"}</div>
                          </td>
                          <td className="px-8 py-5">
                            <div className="flex flex-col gap-1">
                              <div className="flex items-center gap-2">
                                <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700">{log.source}</span>
                                {(log.device_name || (log.source === "Device" && getDeviceName(log.device_id))) && (
                                  <span className="text-[10px] font-bold text-slate-700 dark:text-slate-200 uppercase">{log.device_name || getDeviceName(log.device_id)}</span>
                                )}
                              </div>
                            </div>
                          </td>
                          <td className="px-8 py-5 text-right">
                            <StatusBadge variant="success">{t("attendance_logs.status.recorded")}</StatusBadge>
                          </td>
                        </tr>
                      ))}
                      <InfiniteScrollSentinel sentinelRef={loadMoreRef} isFetchingNextPage={isFetchingNextPage} colSpan={4} />
                    </>
                  )}
                </tbody>
              </table>
            </div>

          </div>
        </>
      )}

      {activeTab === "unknown" && (
        <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead className="sticky top-0 z-10 bg-white/80 dark:bg-slate-900/80 backdrop-blur pb-2">
                <tr className="bg-amber-50/80 dark:bg-amber-950/80 border-b border-amber-100 dark:border-amber-800 shadow-sm backdrop-blur">
                  <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("common.time")}</th>
                  <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("attendance_logs.unknown_id", "Unknown ID")}</th>
                  <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("common.device")}</th>
                  <th className="px-8 py-4 text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest text-right">{t("common.actions")}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                {stagingLoading ? (
                  Array.from({ length: 3 }).map((_, i) => (
                    <tr key={`skeleton-unknown-${i}`} className="animate-pulse border-b border-slate-50 dark:border-slate-800/50">
                      <td className="px-8 py-5">
                        <div className="flex items-center gap-4">
                          <div className="w-10 h-10 rounded-xl bg-amber-50 dark:bg-amber-950/50"></div>
                          <div className="space-y-2">
                            <div className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded"></div>
                            <div className="h-3 w-16 bg-slate-200 dark:bg-slate-800 rounded"></div>
                          </div>
                        </div>
                      </td>
                      <td className="px-8 py-5">
                        <div className="space-y-2">
                          <div className="h-5 w-24 bg-slate-200 dark:bg-slate-800 rounded"></div>
                          <div className="h-3 w-16 bg-slate-200 dark:bg-slate-800 rounded"></div>
                        </div>
                      </td>
                      <td className="px-8 py-5">
                        <div className="h-5 w-20 bg-slate-200 dark:bg-slate-800 rounded"></div>
                      </td>
                      <td className="px-8 py-5 text-right">
                        <div className="flex justify-end gap-2">
                          <div className="h-6 w-16 bg-slate-200 dark:bg-slate-800 rounded-md inline-block"></div>
                          <div className="h-6 w-16 bg-slate-200 dark:bg-slate-800 rounded-md inline-block"></div>
                        </div>
                      </td>
                    </tr>
                  ))
                ) : allStagingLogs.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-8 py-24 text-center">
                      <div className="flex flex-col items-center justify-center space-y-4">
                        <div className="w-16 h-16 bg-emerald-50 dark:bg-emerald-950/30 rounded-full flex items-center justify-center text-emerald-500/50">
                          <Check className="h-8 w-8 text-emerald-400" />
                        </div>
                        <div>
                          <h3 className="text-sm font-bold text-slate-700 dark:text-slate-300 uppercase tracking-widest">{t("attendance_logs.messages.no_unknown_logs", "No unidentified logs")}</h3>
                          <p className="text-xs text-slate-500 dark:text-slate-400 mt-2 font-medium">Awesome! All device punch records have been successfully matched to employees in the system.</p>
                        </div>
                      </div>
                    </td>
                  </tr>
                ) : (
                  <>
                    {allStagingLogs.map((log) => (
                      <tr key={log.id} className="hover:bg-amber-50 dark:bg-amber-950/30 transition-colors group">
                        <td className="px-8 py-5">
                          <div className="flex items-center gap-4">
                            <div className="w-10 h-10 rounded-xl bg-amber-50 dark:bg-amber-950 flex items-center justify-center text-amber-600 dark:text-amber-300 shadow-inner border border-amber-100 dark:border-amber-800/50"><AlertTriangle size={18} /></div>
                            <div>
                              <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
                              <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
                            </div>
                          </div>
                        </td>
                        <td className="px-8 py-5">
                          <div className="font-bold text-amber-700 dark:text-amber-300 text-lg tracking-tight font-mono">{log.raw_user_id}</div>
                          <div className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5">{t("attendance_logs.not_in_system", "Not in system")}</div>
                        </td>
                        <td className="px-8 py-5">
                          <div className="flex flex-col gap-1">
                            {getDeviceName(log.device_id) ? (
                              <span className="text-sm font-bold text-slate-700 dark:text-slate-200">{getDeviceName(log.device_id)}</span>
                            ) : log.device_id ? (
                              <span className="text-xs font-mono text-slate-500 dark:text-slate-400">{log.device_id}</span>
                            ) : (
                              <span className="text-xs text-slate-400 dark:text-slate-500 italic">-</span>
                            )}
                          </div>
                        </td>
                        <td className="px-8 py-5">
                          <div className="flex items-center justify-end gap-2">
                            {assigningLogId === log.id ? (
                              <div className="flex items-center gap-2 min-w-[200px]">
                                <EmployeeSearch
                                  workspaceId={wsId!}
                                  value=""
                                  onChange={(empId) => {
                                    if (empId) {
                                      resolveMutation.mutate({ logId: log.id, employeeId: empId });
                                    }
                                  }}
                                  placeholder={t("attendance_logs.select_employee", "Select employee...")}
                                />
                                <button
                                  onClick={() => setAssigningLogId(null)}
                                  className="p-2 text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600 transition-colors"
                                >
                                  ✕
                                </button>
                              </div>
                            ) : (
                              <>
                                <button
                                  onClick={() => setAssigningLogId(log.id)}
                                  className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-primary-100 dark:bg-primary-900 transition-all"
                                >
                                  <Link2 size={12} /> {t("attendance_logs.assign", "Assign")}
                                </button>
                                <button
                                  onClick={() => handleCreateEmployee(log.raw_user_id)}
                                  className="flex items-center gap-1.5 px-3 py-1.5 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-emerald-100 dark:bg-emerald-900 transition-all"
                                >
                                  <UserPlus size={12} /> {t("attendance_logs.create_employee", "Create")}
                                </button>
                                <button
                                  onClick={() => dismissMutation.mutate(log.id)}
                                  disabled={dismissMutation.isPending}
                                  className="flex items-center gap-1.5 px-3 py-1.5 bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 rounded-lg text-[9px] font-bold uppercase tracking-widest hover:bg-rose-100 dark:bg-rose-900 transition-all disabled:opacity-50"
                                >
                                  <Trash2 size={12} /> {t("common.delete")}
                                </button>
                              </>
                            )}
                          </div>
                        </td>
                      </tr>
                    ))}
                    <InfiniteScrollSentinel sentinelRef={loadMoreStagingRef} isFetchingNextPage={isFetchingNextStagingPage} colSpan={4} />
                  </>
                )}
              </tbody>
            </table>
          </div>
        </div>
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
