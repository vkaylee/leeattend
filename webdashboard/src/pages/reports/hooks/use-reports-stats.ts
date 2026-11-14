
import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { AttendanceReportSort, AttendanceSummary, PaginatedResponse, fetchAttendanceReport } from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { formatHours } from "@/lib/utils/attendance";


interface UseReportsStatsProps {
  dateRange: { start: string; end: string };
  employeeIdsFilter: string[] | undefined;
  employeeStatus?: string;
  selectedDepartmentId: string | null;
  selectedEmployerId: string | null;
  sortBy: AttendanceReportSort | undefined;
}

export function useReportsStats({
  dateRange,
  sortBy,
  employeeIdsFilter,
  selectedDepartmentId,
  employeeStatus,
  selectedEmployerId
}: UseReportsStatsProps) {
  const { data: allStatsResponse } = useQuery<PaginatedResponse<AttendanceSummary>>({
    queryKey: createQueryKey("attendance-report-stats", dateRange, "summary", sortBy, employeeIdsFilter, selectedDepartmentId, employeeStatus, selectedEmployerId),
    queryFn: () => {

      return fetchAttendanceReport({
        start: dateRange.start,
        end: dateRange.end,
        type: "summary",
        page: 1,
        limit: 99999,
        sort: sortBy,
        employeeIds: employeeIdsFilter,
        departmentId: selectedDepartmentId,
        employeeStatus,
        employerId: selectedEmployerId,
      }) as Promise<PaginatedResponse<AttendanceSummary>>;
    },
  });

  const quickStats = useMemo(() => {
    const allData = allStatsResponse?.data ?? [];
    const stdHours = formatHours(
      allData.reduce((acc: number, r: AttendanceSummary) => acc + (r.computed_std_hours || 0), 0)
    );
    const otHours = formatHours(
      allData.reduce((acc: number, r: AttendanceSummary) => acc + (r.computed_ot_hours || 0), 0)
    );

    const totalEmployees = new Set(allData.map((r: AttendanceSummary) => r.employee_id)).size;
    return { stdHours, otHours, totalEmployees };
  }, [allStatsResponse]);

  return { quickStats };
}
