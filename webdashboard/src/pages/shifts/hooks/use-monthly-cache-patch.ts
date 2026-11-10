import type { QueryClient } from "@tanstack/react-query";

import { useCallback } from "react";

import { format } from "date-fns";

import type { EmployeeSchedule } from "@/lib/api/types";

import { fetchSchedules } from "@/lib/api";

interface UseMonthlyCachePatchProps { monthDays: Date[];
queryClient: QueryClient;  }

interface UseMonthlyCachePatchReturn {
  /** Refresh a single employment's schedule row in the monthly query cache. */
  refetchAndPatchRow: (employmentId: string) => Promise<void>;
}

/**
 * Provides `refetchAndPatchRow`, which re-fetches one employment's schedules
 * and splices the fresh `schedules` array into the cached monthly query row,
 * preserving its display fields without invalidating the entire month.
 */
export function useMonthlyRosterCachePatch({
  monthDays,
  queryClient,
}: UseMonthlyCachePatchProps): UseMonthlyCachePatchReturn {
  const monthStartStr =
    monthDays.length > 0 ? format(monthDays[0], "yyyy-MM-dd") : null;
  const monthEndStr =
    monthDays.length > 0
      ? format(monthDays[monthDays.length - 1], "yyyy-MM-dd")
      : null;

  const refetchAndPatchRow = useCallback(
    async (employmentId: string) => {
      if (!monthStartStr || !monthEndStr) return;
      const fresh = await fetchSchedules(monthStartStr, monthEndStr, [
        employmentId,
      ]);
      if (fresh.length === 0) return;
      const freshRow =
        fresh.find((s) => s.employment_id === employmentId) ?? fresh[0];
      queryClient.setQueriesData<EmployeeSchedule[]>(
        {
          predicate: (q) =>
            q.queryKey[0] === "schedules" && q.queryKey[3] === "monthly",
        },
        (old) =>
          old?.map((row) =>
            row.employment_id === employmentId
              ? { ...row, schedules: freshRow.schedules }
              : row,
          ),
      );
    },
    [monthStartStr, monthEndStr, queryClient],
  );

  return { refetchAndPatchRow };
}
