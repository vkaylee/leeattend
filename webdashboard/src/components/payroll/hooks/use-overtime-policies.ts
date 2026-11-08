import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { fetchOvertimePolicies, fetchOvertimePolicy, createOvertimePolicy } from "@/lib/api/payroll";

export function useOvertimePolicies() {
  return useQuery({ queryKey: ["overtime-policies"], queryFn: () => fetchOvertimePolicies(),
 });
}

export function useOvertimePolicy(policyId: string) {
  return useQuery({
    queryKey: ["overtime-policy", policyId],
    queryFn: () => fetchOvertimePolicy(policyId),
    enabled: !!policyId,
  });
}

export function useCreateOvertimePolicy(options?: { onSuccess?: () => void }) {
  const queryClient = useQueryClient();
  const toast = useToast();
  const { t } = useTranslation();

  return useMutation({
    mutationFn: (data: Parameters<typeof createOvertimePolicy>[1]) => createOvertimePolicy(data),
    onSuccess: (_data) => {
      void queryClient.invalidateQueries({ queryKey: ["overtime-policies"] });
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
