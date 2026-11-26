import type { QueryClient } from "@tanstack/react-query";

import { useCallback } from "react";

interface UseMonthlyCachePatchProps {
  monthDays: Date[];
  queryClient: QueryClient;
}

interface UseMonthlyCachePatchReturn {
  /** Invalidate monthly schedule queries so schedules and coverage membership stay in sync. */
  refetchAndPatchRow: (employmentId: string) => Promise<void>;
}

/**
 * Invalidates cached monthly schedule queries after a row mutation so both
 * schedule cells and server-side coverage filter membership refresh cleanly.
 */
export function useMonthlyRosterCachePatch({
  monthDays,
  queryClient,
}: UseMonthlyCachePatchProps): UseMonthlyCachePatchReturn {
  const hasMonth = monthDays.length > 0;

  const refetchAndPatchRow = useCallback(
    async (_employmentId: string) => {
      if (!hasMonth) return;
      await queryClient.invalidateQueries({
        predicate: (q) =>
          q.queryKey[0] === "schedules" && q.queryKey[2] === "monthly",
      });
    },
    [hasMonth, queryClient],
  );

  return { refetchAndPatchRow };
}
