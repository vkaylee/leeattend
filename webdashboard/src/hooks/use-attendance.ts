import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  fetchAttendanceSummaries,
  approveOt,
  enterPaperOt,
  bulkUploadPaperOt,
} from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

export const attendanceKeys = {
  all: ["attendance"] as const,
  summaries: (params: any) => [...attendanceKeys.all, "summaries", params] as const,
};

// --- OT Manager Hooks ---

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
    summaries: query.data || [],
    isLoading: query.isLoading,
    isError: query.isError,
    refetch: query.refetch,
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
      queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
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
      queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
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
      queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
    },
  });

  return {
    bulkUploadPaperOt: mutation.mutateAsync,
    isUploading: mutation.isPending,
  };
}
