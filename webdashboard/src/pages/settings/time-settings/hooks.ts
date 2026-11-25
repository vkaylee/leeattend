import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState, useEffect } from "react";

import { TFunction } from "i18next";

import type { GenericJobStatus } from "@/lib/api/attendance";

import { useToast } from "@/components/ui/toast";
import {
  reprocessAllAttendance,
  checkReprocessStatus,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { maybeNull } from "@/types/api-overrides";

export function useBulkReprocess(t: TFunction) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const [reprocessStart, setReprocessStart] = useState(() => {
    const d = new Date();
    d.setDate(d.getDate() - 30);
    return d.toISOString().split("T")[0];
  });
  const [reprocessEnd, setReprocessEnd] = useState(() => new Date().toISOString().split("T")[0]);
  const [showReprocessConfirm, setShowReprocessConfirm] = useState(false);
  const [pollingJobId, setPollingJobId] = useState<string | null>(null);

  // In-flight guard: prevents double-clicking the bulk "Confirm" button from
  // firing two reprocessAllAttendance jobs before isPending flips true.
  const bulkReprocessInFlight = useRef(false);

  const jobStatusQuery = useQuery({
    queryKey: createQueryKey("generic-job-status", pollingJobId),
    queryFn: () => {
      if (!pollingJobId) return Promise.reject(new Error("No job ID"));
      return checkReprocessStatus(pollingJobId);
    },
    enabled: !!pollingJobId,
    refetchInterval: (query: { state: { data: GenericJobStatus | null | undefined } }) => {
      const data = query.state.data;
      return data && (data.status === "completed" || data.status === "failed") ? false : 3000;
    },
  });

  useEffect(() => {
    if (jobStatusQuery.data) {
      if (jobStatusQuery.data.status === "completed") {
        setTimeout(() => {
          toast.success(t("common.success") + " - Reprocessing finished");
          void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-report") });
          void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-logs") });
          setShowReprocessConfirm(false);
          setPollingJobId(null);
        }, 0);
      } else if (maybeNull(jobStatusQuery.data)?.status === "failed") {
        setTimeout(() => {
          toast.error("Process failed: " + (maybeNull(jobStatusQuery.data)?.error ?? "Unknown error"));
          setShowReprocessConfirm(false);
          setPollingJobId(null);
        }, 0);
      }
    }
  }, [jobStatusQuery.data, queryClient, t, toast]);

  const reprocessMutation = useMutation({
    mutationFn: () => reprocessAllAttendance(reprocessStart, reprocessEnd),
    onMutate: () => {
      bulkReprocessInFlight.current = true;
    },
    onSuccess: (data) => {
      setPollingJobId(data.job_id);
    },
    onError: (e: Error) => {
      toast.error(e.message || t("common.error"));
    },
    onSettled: () => {
      bulkReprocessInFlight.current = false;
    },
  });

  // Lead-edge debounced submit. The "Confirm" button is already disabled while
  // isPending, but this ref blocks the (rare) case where a second click sneaks
  // into the same tick before React Query flips isPending.
  const submitBulkReprocess = () => {
    if (bulkReprocessInFlight.current) return;
    reprocessMutation.mutate();
  };

  return {
    jobStatus: jobStatusQuery.data ?? undefined,
    isPending: reprocessMutation.isPending,
    pollingJobId,
    reprocessEnd,
    reprocessStart,
    submitBulkReprocess,
    setReprocessEnd,
    setReprocessStart,
    setShowReprocessConfirm,
    showReprocessConfirm,
  };
}
