import { useQuery } from "@tanstack/react-query";

import { getCurrentWorkspaceId } from "@/lib/api";
import { fetch } from "@/lib/api/client";

// ── Types ───────────────────────────────────────────────────────────────

export interface CascadeDualRunStatus {
  cutover_at: string | null;
  days_remaining: number | null;
  enabled: boolean;
  message: string;
  window_closes_at: string | null;
  window_phase: "active" | "closing_soon" | "critical" | "final" | "closed";
}

// ── Hook ────────────────────────────────────────────────────────────────

/**
 * ADR-005: Fetch the cascade dual-run window status.
 *
 * Polls every 6 hours — the window changes at most once per day, so
 * aggressive polling would be wasteful.
 */
export function useCascadeDualRun() {
  const wsId = getCurrentWorkspaceId();
  const path = `/v1/workspaces/${wsId}/cascade-dual-run/status`;

  return useQuery({
    queryKey: ["cascade-dual-run", wsId],
    queryFn: async () => {
      const { data } = await fetch<CascadeDualRunStatus>("GET", path);
      return data;
    },
    staleTime: 6 * 60 * 60 * 1000, // 6 hours
    refetchInterval: 6 * 60 * 60 * 1000, // poll every 6 hours
    enabled: !!wsId,
    retry: 1,
  });
}
