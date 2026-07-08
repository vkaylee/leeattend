import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { fetchOvertimePolicies, fetchOvertimePolicy, createOvertimePolicy } from "@/lib/api/payroll";

export function useOvertimePolicies(wsId: string) {
  return useQuery({
    queryKey: ["overtime-policies", wsId],
    queryFn: () => fetchOvertimePolicies(wsId),
    enabled: !!wsId,
  });
}

export function useOvertimePolicy(wsId: string, policyId: string) {
  return useQuery({
    queryKey: ["overtime-policy", policyId],
    queryFn: () => fetchOvertimePolicy(wsId, policyId),
    enabled: !!wsId && !!policyId,
  });
}

export function useCreateOvertimePolicy(wsId: string, options?: { onSuccess?: () => void }) {
  const queryClient = useQueryClient();
  const toast = useToast();
  const { t } = useTranslation();

  return useMutation({
    mutationFn: (data: Parameters<typeof createOvertimePolicy>[1]) => createOvertimePolicy(wsId, data),
    onSuccess: (_data) => {
      void queryClient.invalidateQueries({ queryKey: ["overtime-policies", wsId] });
      toast.success(t("common.success"));
      if (options?.onSuccess) {
        options.onSuccess();
      }
    },
    onError: () => {
      toast.error("Failed to create overtime policy");
    }
  });
}
