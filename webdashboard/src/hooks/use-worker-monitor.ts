import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { FetchMonitorJobsParams, fetchMonitorJobs, fetchMonitorOverview } from "@/lib/api/attendance";

export const MONITOR_REFETCH_INTERVAL_MS = 10_000;
export const MONITOR_REFETCH_INTERVAL_S = MONITOR_REFETCH_INTERVAL_MS / 1000;

type MonitorJobsPageParam = {
  cursor: number | null;
  durable_cursor: string | null;
};

export const monitorKeys = {
  all: ["worker-monitor"] as const,
  jobs: (params: Omit<FetchMonitorJobsParams, "cursor" | "durable_cursor">) => [...monitorKeys.all, "jobs", params] as const,
  overview: () => [...monitorKeys.all, "overview"] as const,
};

export function useWorkerJobs(params: Omit<FetchMonitorJobsParams, "cursor" | "durable_cursor">) {
  return useInfiniteQuery({
    queryKey: monitorKeys.jobs(params),
    queryFn: ({ pageParam }) =>
      fetchMonitorJobs({
        ...params,
        cursor: pageParam.cursor,
        durable_cursor: pageParam.durable_cursor,
      }),
    initialPageParam: { cursor: null, durable_cursor: null } satisfies MonitorJobsPageParam,
    getNextPageParam: (lastPage): MonitorJobsPageParam | undefined => {
      const cursor = lastPage.next_cursor;
      const durableCursor = lastPage.durable_jobs.next_cursor_token;
      return cursor !== null || durableCursor !== null
        ? { cursor, durable_cursor: durableCursor }
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
