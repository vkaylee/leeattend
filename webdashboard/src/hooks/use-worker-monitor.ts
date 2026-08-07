import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { FetchMonitorJobsParams, fetchMonitorJobs, fetchMonitorOverview } from "@/lib/api/attendance";

export const MONITOR_REFETCH_INTERVAL_MS = 10_000;
export const MONITOR_REFETCH_INTERVAL_S = MONITOR_REFETCH_INTERVAL_MS / 1000;

export const monitorKeys = {
  all: ["worker-monitor"] as const,
  jobs: (params: Omit<FetchMonitorJobsParams, "cursor">) => [...monitorKeys.all, "jobs", params] as const,
  overview: () => [...monitorKeys.all, "overview"] as const,
};

export function useWorkerJobs(params: Omit<FetchMonitorJobsParams, "cursor">) {
  return useInfiniteQuery({
    queryKey: monitorKeys.jobs(params),
    queryFn: ({ pageParam }) =>
      fetchMonitorJobs({
        ...params,
        cursor: pageParam,
      }),
    initialPageParam: null as number | null,
    getNextPageParam: (lastPage) => lastPage.next_cursor,
  });
}

export function useWorkerMonitor(sseConnected = false) {
  const query = useQuery({
    queryKey: monitorKeys.overview(),
    queryFn: fetchMonitorOverview,
    refetchInterval: sseConnected ? false : MONITOR_REFETCH_INTERVAL_MS,
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
