import { useQuery } from "@tanstack/react-query";
import { fetchMonitorStats, fetchWorkerList } from "@/lib/api/attendance";
import { getCurrentWorkspaceId } from "@/lib/api";

export const monitorKeys = {
  all: ["worker-monitor"] as const,
  stats: () => [...monitorKeys.all, "stats"] as const,
  workers: () => [...monitorKeys.all, "workers"] as const,
};

export function useWorkerMonitor() {
  const wsId = getCurrentWorkspaceId();

  const statsQuery = useQuery({
    queryKey: monitorKeys.stats(),
    queryFn: () => fetchMonitorStats(wsId!),
    enabled: !!wsId,
    refetchInterval: 5000, // Refresh every 5 seconds for real-time monitoring
  });

  const workersQuery = useQuery({
    queryKey: monitorKeys.workers(),
    queryFn: () => fetchWorkerList(wsId!),
    enabled: !!wsId,
    refetchInterval: 5000,
  });

  return {
    stats: statsQuery.data,
    workers: workersQuery.data ?? [],
    isLoading: statsQuery.isLoading || workersQuery.isLoading,
    isFetching: statsQuery.isFetching || workersQuery.isFetching,
    error: statsQuery.error || workersQuery.error,
    refetch: () => {
      statsQuery.refetch();
      workersQuery.refetch();
    },
  };
}
