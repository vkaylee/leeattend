import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useEffect } from "react";

import { TFunction } from "i18next";

import { useToast } from "@/components/ui/toast";
import {
  reprocessAllAttendance,
  checkReprocessStatus,
  updateDefaultAttendanceRules,
  UpdateAttendanceRulesRequest,
} from "@/lib/api";

export function useTimeSettingsRules(
  wsId: string,
  t: TFunction,
  params: {
    earlyArrivalStrategy: "allow" | "block" | "clamp";
    roundingPolicy: string;
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

  const jobStatusQuery = useQuery({
    queryKey: ["generic-job-status", wsId, pollingJobId],
    queryFn: () => {
      if (!pollingJobId) return Promise.reject(new Error("No job ID"));
      return checkReprocessStatus(wsId, pollingJobId);
    },
    enabled: !!pollingJobId,
    refetchInterval: (query: any) => {
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
      } else if (jobStatusQuery.data.status === "failed") {
        setTimeout(() => {
          toast.error("Process failed: " + (jobStatusQuery.data.error ?? "Unknown error"));
          setShowReprocessConfirm(false);
          setPollingJobId(null);
        }, 0);
      }
    }
  }, [jobStatusQuery.data, queryClient, t, toast]);

  const reprocessMutation = useMutation({
    mutationFn: () => reprocessAllAttendance(wsId, reprocessStart, reprocessEnd),
    onSuccess: (data) => {
      setPollingJobId(data.job_id);
    },
    onError: (e: Error) => {
      toast.error(e.message || t("common.error"));
    },
  });

  return {
    jobStatus: jobStatusQuery.data,
    pollingJobId,
    reprocessEnd,
    reprocessMutation,
    reprocessStart,
    setReprocessEnd,
    setReprocessStart,
    setShowReprocessConfirm,
    showReprocessConfirm,
  };
}
