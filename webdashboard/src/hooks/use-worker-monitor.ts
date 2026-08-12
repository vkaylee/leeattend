import { useInfiniteQuery, useQuery, type InfiniteData } from "@tanstack/react-query";
import { useState } from "react";

import { FetchMonitorJobsParams, fetchMonitorJobs, fetchMonitorOverview } from "@/lib/api/attendance";

export const MONITOR_REFETCH_INTERVAL_MS = 10_000;
export const MONITOR_REFETCH_INTERVAL_S = MONITOR_REFETCH_INTERVAL_MS / 1000;

type MonitorJobsPageParam = {
  cursor: number | null;
  durable_cursor: string | null;
  include_legacy: boolean;
  include_durable: boolean;
};

export const monitorKeys = {
  all: ["worker-monitor"] as const,
  jobs: (params: Omit<FetchMonitorJobsParams, "cursor" | "durable_cursor">) => [...monitorKeys.all, "jobs", params] as const,
  overview: () => [...monitorKeys.all, "overview"] as const,
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
    } as MonitorJobsPageParam,
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
