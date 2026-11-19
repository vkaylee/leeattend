import { useQuery, useInfiniteQuery, useQueryClient, useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { format, endOfMonth } from "date-fns";

import type { RawAttendanceLog, Employee } from "@/lib/api/types";
import type { AttendanceLogsSearch } from "@/pages/attendance/search-schema";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDepartments } from "@/hooks/use-departments";
import { useAllEmployees } from "@/hooks/use-employees";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import {
  fetchRawAttendanceLogs,
  fetchStagingLogs,
  resolveStagingLog,
  dismissStagingLog,
  StagingLog,
  StagingLogsResponse,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { useSearchPatch } from "@/lib/state";
import { Route } from "@/routes/_authenticated/_organization/attendance-logs";


const PAGE_SIZE = 20;

export type TabType = AttendanceLogsSearch["tab"];
export type FilterMode = AttendanceLogsSearch["mode"];

function firstDayOf(offset: number): string {
  const d = new Date();
  d.setDate(1);
  d.setMonth(d.getMonth() + offset);
  return format(new Date(d.getFullYear(), d.getMonth(), 1), "yyyy-MM-dd");
}

function lastDayOf(offset: number): string {
  const d = new Date();
  d.setDate(1);
  d.setMonth(d.getMonth() + offset);
  return format(new Date(d.getFullYear(), d.getMonth() + 1, 0), "yyyy-MM-dd");
}

function resolveDateRange(search: AttendanceLogsSearch): { start: string; end: string } {
  if (search.mode === "custom" && search.from && search.to) {
    return { start: search.from, end: search.to };
  }
  if (search.mode === "specific_month" && search.from && search.to) {
    return { start: search.from, end: search.to };
  }
  const offset = search.mode === "last_month" ? -1 : 0;
  return { start: firstDayOf(offset), end: lastDayOf(offset) };
}

function useAttendanceMutations() {
  const queryClient = useQueryClient();

  const dismissMutation = useMutation({
    mutationFn: (logId: string) => dismissStagingLog(logId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("staging-logs") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("staging-logs-count") });
    },
  });

  const resolveMutation = useMutation({
    mutationFn: ({ logId, employeeId }: { logId: string; employeeId: string }) => resolveStagingLog(logId, employeeId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("staging-logs") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("staging-logs-count") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-raw-logs") });
    },
  });

  return { dismissMutation, resolveMutation };
}


function useAttendanceData(employeeId: string, dateRange: { start: string; end: string }, activeTab: TabType) {
  const { data: recordedPages, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage } = useInfiniteQuery<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }>({
    queryKey: createQueryKey("attendance-raw-logs", employeeId, dateRange),
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
    enabled: activeTab === "recorded",
  });

  const allRecordedLogs = useMemo(() => recordedPages?.pages.flatMap((p) => p.data) ?? [] as RawAttendanceLog[], [recordedPages]);
  const recordedTotal = recordedPages?.pages[0]?.total ?? 0;

  const { data: stagingPages, isLoading: stagingLoading, fetchNextPage: fetchNextStagingPage, hasNextPage: hasNextStagingPage, isFetchingNextPage: isFetchingNextStagingPage } = useInfiniteQuery<StagingLogsResponse>({
    queryKey: createQueryKey("staging-logs"),
    queryFn: ({ pageParam }) => fetchStagingLogs(pageParam as number, PAGE_SIZE),
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: activeTab === "unknown",
  });

  const allStagingLogs = useMemo(() => stagingPages?.pages.flatMap((p) => p.data) ?? [] as StagingLog[], [stagingPages]);
  const stagingTotal = stagingPages?.pages[0]?.total ?? 0;

  const { data: stagingCount } = useQuery({
    queryKey: createQueryKey("staging-logs-count"),
    queryFn: async () => (await fetchStagingLogs(1, 1)).total,
  });

  return { allRecordedLogs, recordedTotal, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage, allStagingLogs, stagingTotal, stagingLoading, stagingCount, isFetchingNextStagingPage, hasNextStagingPage, fetchNextStagingPage };
}

export function useAttendanceLogsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const { formatDate, formatTime } = useDateTimeFormatter();

  const search = Route.useSearch();
  const patchSearch = useSearchPatch<AttendanceLogsSearch>();

  const activeTab = search.tab;
  const employeeId = search.employee ?? "all";
  const filterMode = search.mode;
  const dateRange = useMemo(() => resolveDateRange(search), [search]);
  const specificMonth = useMemo(() => {
    if (search.mode === "specific_month" && search.from) {
      const d = new Date(`${search.from}T00:00:00`);
      return { month: d.getMonth() + 1, year: d.getFullYear() };
    }
    const now = new Date();
    return { month: now.getMonth() + 1, year: now.getFullYear() };
  }, [search.from, search.mode]);

  const setActiveTab = useCallback((tab: TabType) => { patchSearch({ tab }); }, [patchSearch]);
  const setEmployeeId = useCallback((id: string) => { patchSearch({ employee: id === "all" ? undefined : id }); }, [patchSearch]);

  const setDateRange = useCallback(
    (next: { start: string; end: string } | ((prev: { start: string; end: string }) => { start: string; end: string })) => {
      const resolved = typeof next === "function" ? next(dateRange) : next;
      patchSearch({ mode: "custom", from: resolved.start, to: resolved.end });
    },
    [patchSearch, dateRange],
  );

  const handleFilterModeChange = useCallback((mode: FilterMode) => {
    if (mode === "this_month" || mode === "last_month") {
      patchSearch({ mode, from: undefined, to: undefined });
    } else {
      patchSearch({ mode });
    }
  }, [patchSearch]);

  const handleSpecificMonthChange = useCallback((m: number, y: number) => {
    const start = format(new Date(y, m - 1, 1), "yyyy-MM-dd");
    const end = format(endOfMonth(new Date(y, m - 1, 1)), "yyyy-MM-dd");
    patchSearch({ mode: "specific_month", from: start, to: end });
  }, [patchSearch]);

  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  const { data: departments = [] } = useDepartments();

  const { employees: _employees } = useAllEmployees();
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

  const { allRecordedLogs, recordedTotal, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage, allStagingLogs, stagingTotal, stagingLoading, stagingCount, isFetchingNextStagingPage, hasNextStagingPage, fetchNextStagingPage } = useAttendanceData(employeeId, dateRange, activeTab);
  const { dismissMutation, resolveMutation } = useAttendanceMutations();

  const getEmployeeName = useCallback((log: RawAttendanceLog) => log.employee_full_name ?? employeeMap.get(log.employee_id ?? log.employment_id)?.full_name ?? t("common.system"), [employeeMap, t]);
  const getEmployeeID = useCallback((log: RawAttendanceLog) => log.employee_id_number ?? employeeMap.get(log.employee_id ?? log.employment_id)?.employee_id_number ?? "", [employeeMap]);

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({ hasNextPage, isFetchingNextPage, fetchNextPage: () => { void fetchNextPage(); } });
  const loadMoreStagingRef = useInfiniteScroll<HTMLTableRowElement>({ hasNextPage: hasNextStagingPage, isFetchingNextPage: isFetchingNextStagingPage, fetchNextPage: () => { void fetchNextStagingPage(); } });

  return {
    departments, employees: _employees, t, formatDate, formatTime, activeTab, setActiveTab, employeeId, setEmployeeId, filterMode, dateRange, setDateRange, specificMonth, isImportDialogOpen, setIsImportDialogOpen, handleFilterModeChange, handleSpecificMonthChange, allRecordedLogs, recordedTotal, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage, allStagingLogs, stagingTotal, stagingLoading, stagingCount, isFetchingNextStagingPage, hasNextStagingPage, fetchNextStagingPage, getEmployeeName, getEmployeeID, loadMoreRef, loadMoreStagingRef, handleCreateEmployee: (enrollmentId: string) => { void navigate({ to: "/employees/new", search: { enrollment_id: enrollmentId } }); }, dismissMutation, resolveMutation,
  };
}
