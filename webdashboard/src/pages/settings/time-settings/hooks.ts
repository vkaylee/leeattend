import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState, useEffect } from "react";

import { TFunction } from "i18next";

import type { GenericJobStatus } from "@/lib/api/attendance";
import type { UpdateAttendanceRulesRequest } from "@/lib/api/attendance-rules";

import { useToast } from "@/components/ui/toast";
import {
  reprocessAllAttendance,
  checkReprocessStatus,
  updateDefaultAttendanceRules,
} from "@/lib/api";
import { maybeNull } from "@/types/api-overrides";
type RoundingPolicy = "none" | "floor" | "round" | "ceil";

export function useTimeSettingsRules(
  wsId: string,
  t: TFunction,
  params: {
    earlyArrivalStrategy: "allow" | "block" | "clamp";
    roundingPolicy: RoundingPolicy;
    valid_from: string;
    work_unit_minutes: number;
    grace_period_minutes: number;
    onSuccess: () => void;
  }
) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const updateRulesMutation = useMutation({
    mutationFn: (data: UpdateAttendanceRulesRequest) => updateDefaultAttendanceRules(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["attendance-rules", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-policy-versions", wsId] });
      params.onSuccess();
    },
  });

  const handleSaveNewVersion = () => {
    updateRulesMutation.mutate({
      early_arrival_strategy: params.earlyArrivalStrategy,
      rounding_policy: params.roundingPolicy,
      valid_from: params.valid_from,
      work_unit_minutes: params.work_unit_minutes,
      grace_period_minutes: params.grace_period_minutes,
    });
  };

  return { handleSaveNewVersion, isPending: updateRulesMutation.isPending };
}

export function useBulkReprocess(wsId: string, t: TFunction) {
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
    queryKey: ["generic-job-status", wsId, pollingJobId],
    queryFn: () => {
      if (!pollingJobId) return Promise.reject(new Error("No job ID"));
      return checkReprocessStatus(wsId, pollingJobId);
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
          void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
          void queryClient.invalidateQueries({ queryKey: ["attendance-logs"] });
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
    mutationFn: () => reprocessAllAttendance(wsId, reprocessStart, reprocessEnd),
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
