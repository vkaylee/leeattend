import { useInfiniteQuery, useMutation, useQuery, useQueryClient, type InfiniteData } from "@tanstack/react-query";

import {
  acknowledgeMonitorAlert,
  FetchMonitorAlertsParams,
  FetchMonitorJobsParams,
  fetchMonitorAlert,
  fetchMonitorAlerts,
  fetchMonitorJobs,
  fetchMonitorOverview,
  resolveMonitorAlert,
} from "@/lib/api/attendance";

export const MONITOR_REFETCH_INTERVAL_MS = 10_000;
export const MONITOR_REFETCH_INTERVAL_S = MONITOR_REFETCH_INTERVAL_MS / 1000;

interface MonitorJobsPageParam {
  cursor: number | null;
  durable_cursor: string | null;
  include_durable: boolean;
  include_legacy: boolean;
}

export const monitorKeys = {
  all: ["worker-monitor"] as const,
  jobs: (params: Omit<FetchMonitorJobsParams, "cursor" | "durable_cursor">) => [...monitorKeys.all, "jobs", params] as const,
  overview: () => [...monitorKeys.all, "overview"] as const,
  alertsRoot: () => [...monitorKeys.all, "alerts"] as const,
  alerts: (params: FetchMonitorAlertsParams) => [...monitorKeys.alertsRoot(), params] as const,
  alert: (alertId: string) => [...monitorKeys.all, "alert", alertId] as const,
};

export function useWorkerJobs(params: Omit<FetchMonitorJobsParams, "cursor" | "durable_cursor">) {
  return useInfiniteQuery<Awaited<ReturnType<typeof fetchMonitorJobs>>, Error, InfiniteData<Awaited<ReturnType<typeof fetchMonitorJobs>>>, ReturnType<typeof monitorKeys.jobs>, MonitorJobsPageParam>({
    queryKey: monitorKeys.jobs(params),
    queryFn: ({ pageParam }) =>
      fetchMonitorJobs({
        ...params,
        cursor: pageParam.cursor,
        durable_cursor: pageParam.durable_cursor,
        include_legacy: pageParam.include_legacy,
        include_durable: pageParam.include_durable,
      }),
    initialPageParam: {
      cursor: null,
      durable_cursor: null,
      include_legacy: true,
      include_durable: true,
    },
    getNextPageParam: (lastPage): MonitorJobsPageParam | undefined => {
      const cursor = lastPage.next_cursor ?? null;
      const durableCursor = lastPage.durable_jobs.next_cursor_token ?? null;
      const includeLegacy = cursor !== null;
      const includeDurable = durableCursor !== null;
      return includeLegacy || includeDurable
        ? {
          cursor: includeLegacy ? cursor : null,
          durable_cursor: includeDurable ? durableCursor : null,
          include_legacy: includeLegacy,
          include_durable: includeDurable,
        }
        : undefined;
    },
  });
}

export function useMonitorAlerts(params: FetchMonitorAlertsParams = {}) {
  return useQuery({
    queryKey: monitorKeys.alerts(params),
    queryFn: () => fetchMonitorAlerts(params),
    refetchInterval: MONITOR_REFETCH_INTERVAL_MS,
  });
}

export function useMonitorAlert(alertId: string | null) {
  return useQuery({
    queryKey: monitorKeys.alert(alertId ?? ""),
    queryFn: () => fetchMonitorAlert(alertId ?? ""),
    enabled: Boolean(alertId),
  });
}

export function useMonitorAlertMutations() {
  const queryClient = useQueryClient();
  const invalidate = async (alertId: string) => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: monitorKeys.alertsRoot() }),
      queryClient.invalidateQueries({ queryKey: monitorKeys.alert(alertId) }),
    ]);
  };
  const acknowledge = useMutation({
    mutationFn: ({ alertId, reason }: { alertId: string; reason: string }) =>
      acknowledgeMonitorAlert(alertId, { reason }),
    onSuccess: (_data, variables) => invalidate(variables.alertId),
  });
  const resolve = useMutation({
    mutationFn: ({ alertId, reason }: { alertId: string; reason: string }) =>
      resolveMonitorAlert(alertId, { reason }),
    onSuccess: (_data, variables) => invalidate(variables.alertId),
  });
  return { acknowledge, resolve };
}

export function useWorkerMonitor(streamFresh = false) {
  const query = useQuery({
    queryKey: monitorKeys.overview(),
    queryFn: fetchMonitorOverview,
    refetchInterval: streamFresh ? false : MONITOR_REFETCH_INTERVAL_MS,
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
