import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";

import {
  AttendanceEntry,
  AttendanceDailySummary,
  AttendanceParams,
  TimeEntry,
  fetchMyAttendance,
  fetchMyAttendanceState,
  fetchMyAttendanceSummaries,
  fetchMyTimeEntries,
  submitMyAttendanceAction,
  submitMyOtClaim,
  type AttendanceAction,
  type AttendanceActionResponse,
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
  attendanceState: AttendanceActionResponse | undefined;
  entries: TimeEntry[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  isPunching: boolean;
  submitAction: (action: AttendanceAction, note?: string) => Promise<AttendanceActionResponse>;
  refetch: () => Promise<unknown>;
}

export interface UseMyClockOptions {
  enabled?: boolean;
  suppressErrors?: boolean;
}

export function useMyClock({ enabled = true, suppressErrors = false }: UseMyClockOptions = {}): UseMyClockReturn {
  const queryClient = useQueryClient();

  const { startDate, endDate } = useMemo(() => {
    const now = new Date();
    const futureDate = new Date(now.getTime() + 24 * 60 * 60 * 1000);
    const dateOnly = (value: Date) => value.toISOString().slice(0, 10);
    const endDate = dateOnly(futureDate);
    const startDate = dateOnly(new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000));
    return { startDate, endDate };
  }, []);

  const entriesQuery = useQuery<TimeEntry[]>({
    queryKey: essKeys.clockEntries(),
    queryFn: () => fetchMyTimeEntries({ start_date: startDate, end_date: endDate }),
    enabled,
    meta: suppressErrors ? { suppressErrorToast: true } : undefined,
    refetchInterval: enabled ? 30000 : false,
  });

  const stateQuery = useQuery<AttendanceActionResponse>({
    queryKey: createQueryKey(...essKeys.all, "attendance-state"),
    queryFn: fetchMyAttendanceState,
    enabled,
    meta: suppressErrors ? { suppressErrorToast: true } : undefined,
    refetchInterval: enabled ? 30000 : false,
  });

  const activeEntry = stateQuery.data?.entry ?? entriesQuery.data?.find((e) => !e.punch_out_time);

  const actionMutation = useMutation<AttendanceActionResponse, Error, { action: AttendanceAction; note?: string }>({
    mutationFn: ({ action, note }) => submitMyAttendanceAction(action, crypto.randomUUID(), note),
    onSuccess: (response) => {
      queryClient.setQueryData(createQueryKey(...essKeys.all, "attendance-state"), response);
      void queryClient.invalidateQueries({ queryKey: essKeys.all });
    },
  });

  return {
    entries: entriesQuery.data ?? [],
    isLoading: entriesQuery.isLoading || stateQuery.isLoading,
    isError: entriesQuery.isError || stateQuery.isError,
    error: entriesQuery.error ?? stateQuery.error,
    refetch: async () => { await Promise.all([entriesQuery.refetch(), stateQuery.refetch()]); },
    activeEntry,
    attendanceState: stateQuery.data,
    submitAction: (action, note) => actionMutation.mutateAsync({ action, note }),
    isPunching: actionMutation.isPending,
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
