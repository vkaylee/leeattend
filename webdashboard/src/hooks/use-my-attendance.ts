import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";

import {
  AttendanceEntry,
  AttendanceDailySummary,
  AttendanceParams,
  TimeEntry,
  fetchMyAttendance,
  fetchMyAttendanceSummaries,
  fetchMyTimeEntries,
  punchMySelf,
  submitMyOtClaim,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";

import { essKeys } from "./use-ess-keys";


export interface UseMyAttendanceReturn {
  entries: AttendanceEntry[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}

export function useMyAttendance(params?: AttendanceParams): UseMyAttendanceReturn {
  const attendanceQuery = useQuery<AttendanceEntry[]>({
    queryKey: essKeys.attendance(params),
    queryFn: () => fetchMyAttendance(params),
  });

  return {
    entries: attendanceQuery.data ?? [],
    isLoading: attendanceQuery.isLoading,
    isError: attendanceQuery.isError,
    error: attendanceQuery.error,
    refetch: attendanceQuery.refetch,
  };
}

export interface UseMyClockReturn {
  activeEntry: TimeEntry | undefined;
  entries: TimeEntry[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  isPunching: boolean;
  punch: (data?: { note?: string }) => Promise<TimeEntry>;
  refetch: () => Promise<unknown>;
}

export interface UseMyClockOptions {
  enabled?: boolean;
}

export function useMyClock({ enabled = true }: UseMyClockOptions = {}): UseMyClockReturn {
  const queryClient = useQueryClient();

  const { startDate, endDate } = useMemo(() => {
    const now = new Date();
    const futureDate = new Date(now.getTime() + 24 * 60 * 60 * 1000);
    const endDate = futureDate.toISOString();
    const startDate = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString();
    return { startDate, endDate };
  }, []);

  const entriesQuery = useQuery<TimeEntry[]>({
    queryKey: essKeys.clockEntries(),
    queryFn: () => fetchMyTimeEntries({ start_date: startDate, end_date: endDate }),
    enabled,
    refetchInterval: enabled ? 30000 : false,
  });

  const activeEntry = entriesQuery.data?.find((e) => !e.punch_out_time);

  const punchMutation = useMutation<TimeEntry, Error, { note?: string } | undefined>({
    mutationFn: (data?: { note?: string }) =>
      punchMySelf(data ?? {}),
    onSuccess: (newEntry) => {
      queryClient.setQueryData<TimeEntry[]>(essKeys.clockEntries(), (oldData) => {
        if (!oldData) return [newEntry];
        const exists = oldData.some((e) => e.id === newEntry.id);
        if (exists) {
          return oldData.map((e) => (e.id === newEntry.id ? newEntry : e));
        }
        return [newEntry, ...oldData];
      });
      void queryClient.invalidateQueries({ queryKey: essKeys.all });
    },
  });

  return {
    entries: entriesQuery.data ?? [],
    isLoading: entriesQuery.isLoading,
    isError: entriesQuery.isError,
    error: entriesQuery.error,
    refetch: entriesQuery.refetch,
    activeEntry,
    punch: punchMutation.mutateAsync,
    isPunching: punchMutation.isPending,
  };
}

export interface UseMyAttendanceSummariesReturn {
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
  summaries: AttendanceDailySummary[];
}

export function useMyAttendanceSummaries(start: string, end: string): UseMyAttendanceSummariesReturn {
  const query = useQuery<AttendanceDailySummary[]>({
    queryKey: createQueryKey(...essKeys.all, "summaries", start, end),
    queryFn: () => fetchMyAttendanceSummaries(start, end),
  });

  return {
    summaries: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    refetch: query.refetch,
  };
}

export interface UseSubmitOtClaimReturn {
  isSubmitting: boolean;
  submitClaim: (data: { date: string; minutes: number; reason: string }) => Promise<unknown>;
}

export function useSubmitOtClaim(): UseSubmitOtClaimReturn {
  const queryClient = useQueryClient();

  const mutation = useMutation<unknown, Error, { date: string; minutes: number; reason: string }>({
    mutationFn: (data: { date: string; minutes: number; reason: string }) =>
      submitMyOtClaim(data.date, data.minutes, data.reason),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey(...essKeys.all, "summaries") });
    },
  });

  return {
    submitClaim: mutation.mutateAsync,
    isSubmitting: mutation.isPending,
  };
}
