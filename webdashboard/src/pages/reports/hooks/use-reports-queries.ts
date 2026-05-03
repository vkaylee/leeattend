import { useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { AttendanceReportSort, AttendanceSummary, PaginatedResponse, fetchAttendanceReport } from "@/lib/api";

import { useReportsEmployeeSearch } from "./use-reports-employee-search";
import { useReportsMetadata } from "./use-reports-metadata";
import { useReportsStats } from "./use-reports-stats";

const PAGE_SIZE = 50;

interface UseReportsQueriesProps {
  dateRange: { end: string; start: string };
  debouncedSearch: string;
  selectedDepartmentId: string | null;
  sortBy: AttendanceReportSort | undefined;
  workUnitMinutes?: number;
  wsId: string | null;
}

export function useReportsQueries({
  wsId,
  debouncedSearch,
  dateRange,
  sortBy,
  selectedDepartmentId,
  workUnitMinutes = 60
}: UseReportsQueriesProps) {
  const { workspace, departments } = useReportsMetadata({ wsId });
  const { employeeIdsFilter } = useReportsEmployeeSearch({ wsId, debouncedSearch });

  const {
    data: reportPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<AttendanceSummary>>({
    queryKey: ["attendance-report", wsId, dateRange, "summary", PAGE_SIZE, sortBy, employeeIdsFilter, selectedDepartmentId],
    queryFn: ({ pageParam }) => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchAttendanceReport({
        wsId,
        start: dateRange.start,
        end: dateRange.end,
        type: "summary",
        page: pageParam as number,
        limit: PAGE_SIZE,
        sort: sortBy,
        employeeIds: employeeIdsFilter,
        departmentId: selectedDepartmentId,
      }) as Promise<PaginatedResponse<AttendanceSummary>>;
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      const total = lastPage.total || 0;
      if (totalLoaded >= total) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
  });

  const filteredData = useMemo(() => {
    if (!reportPages) return [];
    return reportPages.pages.flatMap((p) => p.data);
  }, [reportPages]);

  const { quickStats } = useReportsStats({
    wsId,
    dateRange,
    sortBy,
    employeeIdsFilter,
    selectedDepartmentId,
    workUnitMinutes
  });

  return {
    workspace,
    departments,
    reportPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    filteredData,
    quickStats
  };
}
