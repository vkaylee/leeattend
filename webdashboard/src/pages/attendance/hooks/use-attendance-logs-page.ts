import { useQuery, useInfiniteQuery, useQueryClient, useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState, useMemo, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { format, endOfMonth } from "date-fns";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { 
  fetchRawAttendanceLogs, 
  getCurrentWorkspaceId, 
  fetchDevices, 
  fetchStagingLogs, 
  resolveStagingLog, 
  dismissStagingLog 
} from "@/lib/api";
import type { RawAttendanceLog, Employee } from "@/lib/api/types";

const PAGE_SIZE = 20;

export type TabType = "recorded" | "unknown";
export type FilterMode = "this_month" | "last_month" | "custom" | "specific_month";

export function useAttendanceLogsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const wsId = getCurrentWorkspaceId();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const queryClient = useQueryClient();

  const searchParams = new URLSearchParams(typeof window !== "undefined" ? window.location.search : "");
  const urlFrom = searchParams.get("from");
  const urlTo = searchParams.get("to");
  const urlEmployee = searchParams.get("employee");

  const [activeTab, setActiveTab] = useState<TabType>("recorded");
  const [employeeId, setEmployeeId] = useState<string>(urlEmployee ?? "all");

  const getMonthRange = (offset = 0) => {
    const d = new Date();
    d.setMonth(d.getMonth() + offset);
    const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), "yyyy-MM-dd");
    const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), "yyyy-MM-dd");
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<FilterMode>(
    urlFrom && urlTo ? "custom" : "this_month"
  );
  const [dateRange, setDateRange] = useState(
    urlFrom && urlTo ? { start: urlFrom, end: urlTo } : getMonthRange(0)
  );
  const [specificMonth, setSpecificMonth] = useState({ 
    month: new Date().getMonth() + 1, 
    year: new Date().getFullYear() 
  });
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  useEffect(() => {
    if (urlFrom || urlTo || urlEmployee) {
      const url = new URL(window.location.href);
      url.searchParams.delete("from");
      url.searchParams.delete("to");
      url.searchParams.delete("employee");
      window.history.replaceState({}, "", url.toString());
    }
  }, [urlFrom, urlTo, urlEmployee]);

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

  const { employees: _employees } = useAllEmployees(wsId);

  const employeeMap = useMemo(() => {
    const map = new Map<string, Employee>();
    _employees.forEach((emp) => map.set(emp.id, emp));
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
      return fetchRawAttendanceLogs({ 
        employee_id: employeeId === "all" ? undefined : employeeId, 
        from, 
        to, 
        page: pageParam as number, 
        limit: PAGE_SIZE 
      });
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
    if (!recordedPages) return [] as RawAttendanceLog[];
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
    queryFn: ({ pageParam }) => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchStagingLogs(wsId, pageParam as number, PAGE_SIZE);
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      if (totalLoaded >= (lastPage.total || 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab === "unknown",
  });

  const allStagingLogs = useMemo(() => {
    if (!stagingPages) return [] as any[]; // Using any temporarily if StagingLog is problematic, but let's try to be specific
    return stagingPages.pages.flatMap((p) => p.data);
  }, [stagingPages]);

  const stagingTotal = stagingPages?.pages[0]?.total ?? 0;

  const { data: stagingCount } = useQuery({
    queryKey: ["staging-logs-count", wsId],
    queryFn: async () => {
      if (!wsId) return 0;
      const result = await fetchStagingLogs(wsId, 1, 1);
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
    mutationFn: (logId: string) => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return dismissStagingLog(wsId, logId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      void queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
    },
  });

  const resolveMutation = useMutation({
    mutationFn: ({ logId, employeeId }: { logId: string; employeeId: string }) => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return resolveStagingLog(wsId, logId, employeeId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staging-logs"] });
      void queryClient.invalidateQueries({ queryKey: ["staging-logs-count"] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-raw-logs"] });
    },
  });

  const getEmployeeName = useCallback((log: RawAttendanceLog) => {
    if (log.employee_full_name) return log.employee_full_name;
    const emp = employeeMap.get(log.employee_id);
    return emp ? emp.full_name : t("common.system");
  }, [employeeMap, t]);

  const getEmployeeID = useCallback((log: RawAttendanceLog) => {
    if (log.employee_id_number) return log.employee_id_number;
    const emp = employeeMap.get(log.employee_id);
    return emp ? emp.employee_id_number : "";
  }, [employeeMap]);

  const deviceMap = useMemo(() => {
    const map = new Map<string, string>();
    if (devicesData) {
      devicesData.forEach((d) => {
        if (d && typeof d === 'object' && 'id' in d && 'name' in d) {
          map.set(d.id as string, d.name as string);
        }
      });
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

  return {
    wsId,
    t,
    formatDate,
    formatTime,
    activeTab,
    setActiveTab,
    employeeId,
    setEmployeeId,
    filterMode,
    dateRange,
    setDateRange,
    specificMonth,
    isImportDialogOpen,
    setIsImportDialogOpen,
    handleFilterModeChange,
    handleSpecificMonthChange,
    allRecordedLogs,
    recordedTotal,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
    allStagingLogs,
    stagingTotal,
    stagingLoading,
    stagingCount,
    isFetchingNextStagingPage,
    hasNextStagingPage,
    fetchNextStagingPage,
    getEmployeeName,
    getEmployeeID,
    getDeviceName,
    loadMoreRef,
    loadMoreStagingRef,
    handleCreateEmployee,
    dismissMutation,
    resolveMutation,
  };
}
