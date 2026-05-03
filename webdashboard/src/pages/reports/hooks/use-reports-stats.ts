import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { AttendanceReportSort, AttendanceSummary, PaginatedResponse, fetchAttendanceReport } from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";

interface UseReportsStatsProps {
  dateRange: { start: string; end: string };
  employeeIdsFilter: string[] | undefined;
  selectedDepartmentId: string | null;
  sortBy: AttendanceReportSort | undefined;
  workUnitMinutes: number;
  wsId: string | null;
}

export function useReportsStats({
  wsId,
  dateRange,
  sortBy,
  employeeIdsFilter,
  selectedDepartmentId,
  workUnitMinutes
}: UseReportsStatsProps) {
  const { data: allStatsResponse } = useQuery<PaginatedResponse<AttendanceSummary>>({
    queryKey: ["attendance-report-stats", wsId, dateRange, "summary", sortBy, employeeIdsFilter, selectedDepartmentId],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchAttendanceReport({
        wsId,
        start: dateRange.start,
        end: dateRange.end,
        type: "summary",
        page: 1,
        limit: 99999,
        sort: sortBy,
        employeeIds: employeeIdsFilter,
        departmentId: selectedDepartmentId,
      }) as Promise<PaginatedResponse<AttendanceSummary>>;
    },
    enabled: !!wsId,
  });

  const quickStats = useMemo(() => {
    const allData = allStatsResponse?.data ?? [];
    const stdHours = formatUnitsToHours(
      allData.reduce((acc: number, r: AttendanceSummary) => acc + (r.total_std_units || 0), 0),
      workUnitMinutes
    );
    const otHours = formatUnitsToHours(
      allData.reduce((acc: number, r: AttendanceSummary) => acc + (r.total_ot_units || 0), 0),
      workUnitMinutes
    );

    const totalEmployees = new Set(allData.map((r: AttendanceSummary) => r.employee_id)).size;
    return { stdHours, otHours, totalEmployees };
  }, [allStatsResponse, workUnitMinutes]);

  return { quickStats };
}
