import { useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import type { AttendanceDailySummary, PaginatedResponse } from "@/lib/api";

import { useOrganizationQuery, useOrganizationMutation } from "@/hooks/common/use-workspace-query";
import {
  fetchAttendanceSummaries,
  fetchAttendanceSummariesPaginated,
  approveOt,
  enterPaperOt,
  bulkUploadPaperOt,
  claimOt,
} from "@/lib/api";
import { getStandaloneOrganizationId } from "@/lib/api/helpers";
import { maybeNull } from "@/types/api-overrides";

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
  const query = useOrganizationQuery<AttendanceDailySummary[]>({
    queryKey: attendanceKeys.summaries({ start, end, employeeId }),
    queryFn: (wsId) => fetchAttendanceSummaries(wsId, start, end, employeeId),
  });

  return {
    summaries: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    refetch: query.refetch,
  };
}

export function useAttendanceSummariesInfinite(
  start: string,
  end: string,
  options?: { employeeId?: string; otStatus?: string },
) {
  const wsId = getStandaloneOrganizationId();

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
        return sum + (maybeNull(pageData)?.length ?? 0);
      }, 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
  });

  const summaries = useMemo(
    () => infiniteQuery.data?.pages.flatMap((p) => Array.isArray(p.data) ? p.data : []) ?? [],
    [infiniteQuery.data],
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
  const mutation = useOrganizationMutation<
    { employeeId: string; date: string; approved_minutes: number; manager_note?: string; status: "approved" | "rejected" }
  >({
    mutationFn: (wsId, data) =>
      approveOt(wsId, data.employeeId, {
        date: data.date,
        approved_minutes: data.approved_minutes,
        manager_note: data.manager_note,
        status: data.status,
      }),
    keys: [attendanceKeys.all],
  });

  return {
    approveOt: mutation.mutateAsync,
    isProcessing: mutation.isPending,
  };
}

export function useEnterPaperOt() {
  const mutation = useOrganizationMutation<
    {
      employeeId: string;
      date: string;
      claimed_minutes: number;
      reason: string;
      paper_form_photo_url?: string;
    },
    AttendanceDailySummary
  >({
    mutationFn: (wsId, data) =>
      enterPaperOt(wsId, data.employeeId, {
        date: data.date,
        claimed_minutes: data.claimed_minutes,
        reason: data.reason,
        paper_form_photo_url: data.paper_form_photo_url,
      }),
    keys: [attendanceKeys.all],
  });

  return {
    enterPaperOt: mutation.mutateAsync,
    isSubmitting: mutation.isPending,
  };
}

export function useBulkUploadPaperOt() {
  const mutation = useOrganizationMutation<
    File,
    { total_rows: number; success_count: number; failed_count: number; errors: { line_number: number; employment_id: string; error_message: string }[] }
  >({
    mutationFn: (wsId, file) => bulkUploadPaperOt(wsId, file),
    keys: [attendanceKeys.all],
  });

  return {
    bulkUploadPaperOt: mutation.mutateAsync,
    isUploading: mutation.isPending,
  };
}

export function useClaimOt() {
  const mutation = useOrganizationMutation<
    { employeeId: string; date: string; claimed_minutes: number; reason?: string }
  >({
    mutationFn: (wsId, data) =>
      claimOt(wsId, data.employeeId, {
        date: data.date,
        claimed_minutes: data.claimed_minutes,
        reason: data.reason,
      }),
    keys: [attendanceKeys.all, ["attendance-report"]],
  });

  return {
    claimOt: mutation.mutateAsync,
    isSubmitting: mutation.isPending,
  };
}
