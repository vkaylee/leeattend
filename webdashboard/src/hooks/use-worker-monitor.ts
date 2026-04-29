import { useQuery } from "@tanstack/react-query";

import { fetchMonitorOverview } from "@/lib/api/attendance";

export const MONITOR_REFETCH_INTERVAL_MS = 2000;
export const MONITOR_REFETCH_INTERVAL_S = MONITOR_REFETCH_INTERVAL_MS / 1000;

export const monitorKeys = {
  all: ["worker-monitor"] as const,
  overview: () => [...monitorKeys.all, "overview"] as const,
};

export function useWorkerMonitor() {
  const query = useQuery({
    queryKey: monitorKeys.overview(),
    queryFn: fetchMonitorOverview,
    refetchInterval: MONITOR_REFETCH_INTERVAL_MS,
  });

  return {
    stats: query.data,
    workers: query.data?.workers ?? [],
    recentJobs: query.data?.recent_jobs ?? [],
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    error: query.error,
    refetch: query.refetch,
  };
}
