
import { useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { AttendanceReportSort, AttendanceSummary, PaginatedResponse, fetchAttendanceReport } from "@/lib/api";

import { useReportsEmployeeSearch } from "./use-reports-employee-search";
import { useReportsMetadata } from "./use-reports-metadata";
import { useReportsStats } from "./use-reports-stats";
import { createQueryKey } from "@/lib/query-keys";


const PAGE_SIZE = 50;

interface UseReportsQueriesProps {
  dateRange: { end: string; start: string };
  debouncedSearch: string;
  employeeStatus?: string;
  selectedDepartmentId: string | null;
  selectedEmployerId: string | null;
  sortBy: AttendanceReportSort | undefined;
}

export function useReportsQueries({
  debouncedSearch,
  dateRange,
  sortBy,
  selectedDepartmentId,
  employeeStatus,
  selectedEmployerId,
}: UseReportsQueriesProps) {
  const { organization, departments, employers } = useReportsMetadata();
  const { employeeIdsFilter } = useReportsEmployeeSearch({ debouncedSearch });

  const {
    data: reportPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<AttendanceSummary>>({
    queryKey: createQueryKey("attendance-report", dateRange, "summary", PAGE_SIZE, sortBy, employeeIdsFilter, selectedDepartmentId, employeeStatus, selectedEmployerId),
    queryFn: ({ pageParam }) => {

      return fetchAttendanceReport({
        start: dateRange.start,
        end: dateRange.end,
        type: "summary",
        page: pageParam as number,
        limit: PAGE_SIZE,
        sort: sortBy,
        employeeIds: employeeIdsFilter,
        departmentId: selectedDepartmentId,
        employeeStatus,
        employerId: selectedEmployerId,
      }) as Promise<PaginatedResponse<AttendanceSummary>>;
    },
    getNextPageParam: (lastPage) => {
      if (lastPage.page * lastPage.limit >= lastPage.total) return undefined;
      return lastPage.page + 1;
    },
    initialPageParam: 1,
  });

  const filteredData = useMemo(() => {
    if (!reportPages) return [];
    return reportPages.pages.flatMap((p) => p.data);
  }, [reportPages]);

  const { quickStats } = useReportsStats({
    dateRange,
    sortBy,
    employeeIdsFilter,
    selectedDepartmentId,
    employeeStatus,
    selectedEmployerId,
  });

  return {
    organization,
    departments,
    employers,
    reportPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    filteredData,
    quickStats
  };
}
