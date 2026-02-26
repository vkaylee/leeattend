import { useQuery } from "@tanstack/react-query";
import { fetchMonitorOverview } from "@/lib/api/attendance";

export const monitorKeys = {
  all: ["worker-monitor"] as const,
  overview: () => [...monitorKeys.all, "overview"] as const,
};

export function useWorkerMonitor() {
  const query = useQuery({
    queryKey: monitorKeys.overview(),
    queryFn: fetchMonitorOverview,
    refetchInterval: 5000, // Refresh every 5 seconds for real-time monitoring
  });

  return {
    stats: query.data,
    workers: query.data?.workers ?? [],
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    error: query.error,
    refetch: query.refetch,
  };
}
