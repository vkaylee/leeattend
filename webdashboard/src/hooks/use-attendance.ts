import { useQuery, useMutation, useInfiniteQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";

import type { AttendanceDailySummary, PaginatedResponse } from "@/lib/api";

import {
  fetchAttendanceSummaries,
  fetchAttendanceSummariesPaginated,
  approveOt,
  enterPaperOt,
  bulkUploadPaperOt,
  claimOt,
} from "@/lib/api";
import { maybeNull } from "@/types/api-overrides";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

interface SummaryParams {
  employeeId?: string;
  end: string;
  start: string;
}

export const attendanceKeys = {
  all: ["attendance"] as const,
  summaries: (params: SummaryParams) => [...attendanceKeys.all, "summaries", params] as const,
};

// --- OT Manager Hooks ---

const POST_OT_PAGE_SIZE = 20;

export function useAttendanceSummaries(start: string, end: string, employeeId?: string) {
  const wsId = getCurrentWorkspaceId();

  const query = useQuery({
    queryKey: attendanceKeys.summaries({ start, end, employeeId }),
    queryFn: () => {
        if (!wsId) throw new Error("No workspace");
        return fetchAttendanceSummaries(wsId, start, end, employeeId);
    },
    enabled: !!wsId,
  });

  return {
    summaries: (query.data ?? []),
    isLoading: query.isLoading,
    isError: query.isError,
    refetch: query.refetch,
  };
}

export function useAttendanceSummariesInfinite(
  start: string,
  end: string,
  options?: { employeeId?: string; otStatus?: string }
) {
  const wsId = getCurrentWorkspaceId();

  const infiniteQuery = useInfiniteQuery<PaginatedResponse<AttendanceDailySummary>>({
    queryKey: [...attendanceKeys.all, "summaries", { start, end, employeeId: options?.employeeId, otStatus: options?.otStatus }],
    queryFn: ({ pageParam }) => {
      if (!wsId) throw new Error("No workspace");
      return fetchAttendanceSummariesPaginated(wsId, start, end, {
        employeeId: options?.employeeId,
        page: pageParam as number,
        limit: POST_OT_PAGE_SIZE,
        otStatus: options?.otStatus,
      });
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => {
        const pageData = p.data;
        return sum + (maybeNull(pageData)?.length || 0);
      }, 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
  });

  const summaries = useMemo(
    () => infiniteQuery.data?.pages.flatMap((p) => Array.isArray(p.data) ? p.data : []) ?? [],
    [infiniteQuery.data]
  );

  return {
    summaries,
    isLoading: infiniteQuery.isLoading,
    isError: infiniteQuery.isError,
    fetchNextPage: infiniteQuery.fetchNextPage,
    hasNextPage: infiniteQuery.hasNextPage,
    isFetchingNextPage: infiniteQuery.isFetchingNextPage,
  };
}

export function useApproveOt() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();

  const mutation = useMutation({
    mutationFn: (data: { employeeId: string, date: string; approved_minutes: number; manager_note?: string; status: "approved" | "rejected" }) => {
        if (!wsId) throw new Error("No workspace");
        return approveOt(wsId, data.employeeId, {
            date: data.date,
            approved_minutes: data.approved_minutes,
            manager_note: data.manager_note,
            status: data.status,
        });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
    },
  });

  return {
    approveOt: mutation.mutateAsync,
    isProcessing: mutation.isPending,
  };
}

export function useEnterPaperOt() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();

  const mutation = useMutation({
    mutationFn: (data: {
      employeeId: string;
      date: string;
      claimed_minutes: number;
      reason: string;
      paper_form_photo_url?: string;
    }) => {
      if (!wsId) throw new Error("No workspace");
      return enterPaperOt(wsId, data.employeeId, {
        date: data.date,
        claimed_minutes: data.claimed_minutes,
        reason: data.reason,
        paper_form_photo_url: data.paper_form_photo_url,
      });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
    },
  });

  return {
    enterPaperOt: mutation.mutateAsync,
    isSubmitting: mutation.isPending,
  };
}

export function useBulkUploadPaperOt() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();

  const mutation = useMutation({
    mutationFn: (file: File) => {
      if (!wsId) throw new Error("No workspace");
      return bulkUploadPaperOt(wsId, file);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
    },
  });

  return {
    bulkUploadPaperOt: mutation.mutateAsync,
    isUploading: mutation.isPending,
  };
}

export function useClaimOt() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();

  const mutation = useMutation({
    mutationFn: (data: { employeeId: string; date: string; claimed_minutes: number; reason?: string }) => {
      if (!wsId) throw new Error("No workspace");
      return claimOt(wsId, data.employeeId, {
        date: data.date,
        claimed_minutes: data.claimed_minutes,
        reason: data.reason,
      });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
      void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
    },
  });

  return {
    claimOt: mutation.mutateAsync,
    isSubmitting: mutation.isPending,
  };
}
