import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { fetchAttendanceReport, type DailyAttendanceReport } from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";


export interface UseMonthlyTimesheetDataParams {
  enabled?: boolean;
  month: number;
  year: number;
}

export function formatDayKey(year: number, month: number, day: number): string {
  const m = String(month).padStart(2, "0");
  const d = String(day).padStart(2, "0");
  return `${year}-${m}-${d}`;
}

export function makeTimesheetCellKey(employeeId: string, dateStr: string): string {
  return `${employeeId}:${dateStr}`;
}

export function useMonthlyTimesheetData({
  month,
  year,
  enabled = true,
}: UseMonthlyTimesheetDataParams) {
  const lastDay = useMemo(() => new Date(year, month, 0).getDate(), [year, month]);
  const startDate = useMemo(() => formatDayKey(year, month, 1), [year, month]);
  const endDate = useMemo(() => formatDayKey(year, month, lastDay), [year, month, lastDay]);

  const query = useQuery({
    queryKey: createQueryKey("monthly-timesheet-daily", year, month),
    queryFn: async () => {
      const res = await fetchAttendanceReport({
        start: startDate,
        end: endDate,
        type: "daily",
        limit: 10000,
      });

      const list = (res && typeof res === "object" && "data" in res && Array.isArray(res.data))
        ? (res.data as DailyAttendanceReport[])
        : Array.isArray(res)
          ? (res as DailyAttendanceReport[])
          : [];

      return list;
    },
    enabled,
    staleTime: 5 * 60 * 1000, // 5 minutes cache
  });

  const dailyReports = useMemo(() => query.data ?? [], [query.data]);

  // O(1) Lookup Map: Key = `${employee_id}:${date_str}`
  const attendanceMap = useMemo(() => {
    const map = new Map<string, DailyAttendanceReport>();
    for (const report of dailyReports) {
      if (report.employee_id && report.date) {
        // Strip time if ISO string
        const dateKey = report.date.split("T")[0];
        map.set(makeTimesheetCellKey(report.employee_id, dateKey), report);
      }
    }
    return map;
  }, [dailyReports]);

  return {
    attendanceMap,
    dailyReports,
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    refetch: query.refetch,
  };
}
