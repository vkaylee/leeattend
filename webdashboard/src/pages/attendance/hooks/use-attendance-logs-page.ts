import { useQuery, useInfiniteQuery, useQueryClient, useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState, useMemo, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { format, endOfMonth } from "date-fns";

import type { RawAttendanceLog, Employee } from "@/lib/api/types";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { 
  fetchRawAttendanceLogs, 
  getCurrentWorkspaceId, 
  fetchDevices, 
  fetchStagingLogs, 
  resolveStagingLog, 
  dismissStagingLog,
  StagingLog,
  StagingLogsResponse
} from "@/lib/api";

const PAGE_SIZE = 20;

export type TabType = "recorded" | "unknown";
export type FilterMode = "this_month" | "last_month" | "custom" | "specific_month";

function useAttendanceDateRange(urlFrom: string | null, urlTo: string | null) {
  const getMonthRange = (offset = 0) => {
    const d = new Date();
    d.setDate(1); // Prevent rollover when today is 31st
    d.setMonth(d.getMonth() + offset);
    const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), "yyyy-MM-dd");
    const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), "yyyy-MM-dd");
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<FilterMode>(urlFrom && urlTo ? "custom" : "this_month");
  const [dateRange, setDateRange] = useState(urlFrom && urlTo ? { start: urlFrom, end: urlTo } : getMonthRange(0));
  const [specificMonth, setSpecificMonth] = useState({ month: new Date().getMonth() + 1, year: new Date().getFullYear() });

  const handleFilterModeChange = useCallback((mode: FilterMode) => {
    setFilterMode(mode);
    if (mode === "this_month") setDateRange(getMonthRange(0));
    else if (mode === "last_month") setDateRange(getMonthRange(-1));
  }, []);

  const handleSpecificMonthChange = useCallback((m: number, y: number) => {
    setSpecificMonth({ month: m, year: y });
    setFilterMode("specific_month");
    const start = format(new Date(y, m - 1, 1), "yyyy-MM-dd");
    const end = format(endOfMonth(new Date(y, m - 1, 1)), "yyyy-MM-dd");
    setDateRange({ start, end });
  }, []);

  return { dateRange, filterMode, handleFilterModeChange, handleSpecificMonthChange, setDateRange, specificMonth };
}

function useAttendanceMutations(wsId: string | null) {
  const queryClient = useQueryClient();

  const dismissMutation = useMutation({
    mutationFn: (logId: string) => wsId ? dismissStagingLog(wsId, logId) : Promise.reject(new Error("No workspace ID")),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      void queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
    },
  });

  const resolveMutation = useMutation({
    mutationFn: ({ logId, employeeId }: { logId: string; employeeId: string }) => wsId ? resolveStagingLog(wsId, logId, employeeId) : Promise.reject(new Error("No workspace ID")),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      void queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-raw-logs"] });
    },
  });

  return { dismissMutation, resolveMutation };
}

function useAttendanceDevices(wsId: string | null) {
  const { data: devicesData } = useQuery({ queryKey: ["devices", wsId], queryFn: () => fetchDevices(), enabled: !!wsId });
  const deviceMap = useMemo(() => {
    const map = new Map<string, string>();
    devicesData?.forEach((d) => { if ('id' in d && 'name' in d) map.set(d.id, d.name); });
    return map;
  }, [devicesData]);

  const getDeviceName = useCallback((deviceId?: string | null): string => deviceId ? deviceMap.get(deviceId) ?? "" : "", [deviceMap]);
  return { getDeviceName };
}

function useAttendanceData(wsId: string | null, employeeId: string, dateRange: { start: string; end: string }, activeTab: TabType) {
  const { data: recordedPages, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage } = useInfiniteQuery<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }>({
    queryKey: ["attendance-raw-logs", wsId, employeeId, dateRange],
    queryFn: ({ pageParam }) => {
      const from = dateRange.start ? new Date(`${dateRange.start}T00:00:00`).toISOString() : undefined;
      const to = dateRange.end ? new Date(`${dateRange.end}T23:59:59`).toISOString() : undefined;
      return fetchRawAttendanceLogs({ employee_id: employeeId === "all" ? undefined : employeeId, from, to, page: pageParam as number, limit: PAGE_SIZE });
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab === "recorded",
  });

  const allRecordedLogs = useMemo(() => recordedPages?.pages.flatMap((p) => p.data) ?? [] as RawAttendanceLog[], [recordedPages]);
  const recordedTotal = recordedPages?.pages[0]?.total ?? 0;

  const { data: stagingPages, isLoading: stagingLoading, fetchNextPage: fetchNextStagingPage, hasNextPage: hasNextStagingPage, isFetchingNextPage: isFetchingNextStagingPage } = useInfiniteQuery<StagingLogsResponse>({
    queryKey: ["staging-logs", wsId],
    queryFn: ({ pageParam }) => wsId ? fetchStagingLogs(wsId, pageParam as number, PAGE_SIZE) : Promise.reject(new Error("No workspace ID")),
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab === "unknown",
  });

  const allStagingLogs = useMemo(() => stagingPages?.pages.flatMap((p) => p.data) ?? [] as StagingLog[], [stagingPages]);
  const stagingTotal = stagingPages?.pages[0]?.total ?? 0;

  const { data: stagingCount } = useQuery({
    queryKey: ["staging-logs-count", wsId],
    queryFn: async () => wsId ? (await fetchStagingLogs(wsId, 1, 1)).total : 0,
    enabled: !!wsId,
  });

  return { allRecordedLogs, recordedTotal, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage, allStagingLogs, stagingTotal, stagingLoading, stagingCount, isFetchingNextStagingPage, hasNextStagingPage, fetchNextStagingPage };
}

export function useAttendanceLogsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const wsId = getCurrentWorkspaceId();
  const { formatDate, formatTime } = useDateTimeFormatter();

  const searchParams = new URLSearchParams(typeof window !== "undefined" ? window.location.search : "");
  const urlFrom = searchParams.get("from");
  const urlTo = searchParams.get("to");
  const urlEmployee = searchParams.get("employee");

  const [activeTab, setActiveTab] = useState<TabType>("recorded");
  const [employeeId, setEmployeeId] = useState<string>(urlEmployee ?? "all");
  const { dateRange, filterMode, handleFilterModeChange, handleSpecificMonthChange, setDateRange, specificMonth } = useAttendanceDateRange(urlFrom, urlTo);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  useEffect(() => {
    if (urlFrom || urlTo || urlEmployee) {
      const url = new URL(window.location.href);
      url.searchParams.delete("from"); url.searchParams.delete("to"); url.searchParams.delete("employee");
      window.history.replaceState({}, "", url.toString());
    }
  }, [urlFrom, urlTo, urlEmployee]);

  const { employees: _employees } = useAllEmployees(wsId);
  const employeeMap = useMemo(() => {
    const map = new Map<string, Employee>();
    _employees.forEach((emp) => {
      map.set(emp.id, emp);
      if (emp.active_employment_id) {
        map.set(emp.active_employment_id, emp);
      }
    });
    return map;
  }, [_employees]);

  const { allRecordedLogs, recordedTotal, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage, allStagingLogs, stagingTotal, stagingLoading, stagingCount, isFetchingNextStagingPage, hasNextStagingPage, fetchNextStagingPage } = useAttendanceData(wsId, employeeId, dateRange, activeTab);
  const { dismissMutation, resolveMutation } = useAttendanceMutations(wsId);
  const { getDeviceName } = useAttendanceDevices(wsId);

  const getEmployeeName = useCallback((log: RawAttendanceLog) => log.employee_full_name ?? employeeMap.get(log.employee_id || log.employment_id)?.full_name ?? t("common.system"), [employeeMap, t]);
  const getEmployeeID = useCallback((log: RawAttendanceLog) => log.employee_id_number ?? employeeMap.get(log.employee_id || log.employment_id)?.employee_id_number ?? "", [employeeMap]);

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({ hasNextPage, isFetchingNextPage, fetchNextPage: () => { void fetchNextPage(); } });
  const loadMoreStagingRef = useInfiniteScroll<HTMLTableRowElement>({ hasNextPage: hasNextStagingPage, isFetchingNextPage: isFetchingNextStagingPage, fetchNextPage: () => { void fetchNextStagingPage(); } });

  return {
    wsId, t, formatDate, formatTime, activeTab, setActiveTab, employeeId, setEmployeeId, filterMode, dateRange, setDateRange, specificMonth, isImportDialogOpen, setIsImportDialogOpen, handleFilterModeChange, handleSpecificMonthChange, allRecordedLogs, recordedTotal, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage, allStagingLogs, stagingTotal, stagingLoading, stagingCount, isFetchingNextStagingPage, hasNextStagingPage, fetchNextStagingPage, getEmployeeName, getEmployeeID, getDeviceName, loadMoreRef, loadMoreStagingRef, handleCreateEmployee: (enrollmentId: string) => { void navigate({ to: "/employees", search: { enrollment_id: enrollmentId } }); }, dismissMutation, resolveMutation,
  };
}
