import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
  fetchOvertimePolicies,
  fetchOvertimePolicy,
  createOvertimePolicy,
  createOvertimePolicyVersion,
  patchOvertimePolicyVersion,
  publishOvertimePolicyVersion,
} from "@/lib/api/payroll";
import { createQueryKey } from "@/lib/query-keys";

export function useOvertimePolicies() {
  return useQuery({
    queryKey: createQueryKey("overtime-policies"),
    queryFn: () => fetchOvertimePolicies(),
  });
}

export function useOvertimePolicy(policyId: string) {
  return useQuery({
    queryKey: createQueryKey("overtime-policy", policyId),
    queryFn: () => fetchOvertimePolicy(policyId),
    enabled: Boolean(policyId),
  });
}

export function useCreateOvertimePolicy(options?: { onSuccess?: () => void }) {
  const queryClient = useQueryClient();
  const toast = useToast();
  const { t } = useTranslation();

  return useMutation({
    mutationFn: (data: Parameters<typeof createOvertimePolicy>[0]) => createOvertimePolicy(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("overtime-policies") });
      toast.success(t("common.success"));
      options?.onSuccess?.();
    },
    onError: () => {
      toast.error(t("common.error", "Failed to create overtime policy"));
    },
  });
}

export function useOvertimePolicyVersionMutations(policyId: string) {
  const queryClient = useQueryClient();
  const toast = useToast();
  const { t } = useTranslation();

  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: createQueryKey("overtime-policies") });
    void queryClient.invalidateQueries({ queryKey: createQueryKey("overtime-policy", policyId) });
  };

  const createVersion = useMutation({
    mutationFn: (data: Parameters<typeof createOvertimePolicyVersion>[1]) => createOvertimePolicyVersion(policyId, data),
    onSuccess: () => { invalidate(); toast.success(t("common.success")); },
  });

  const patchVersion = useMutation({
    mutationFn: ({ versionId, data }: { data: Parameters<typeof patchOvertimePolicyVersion>[2]; versionId: string }) =>
      patchOvertimePolicyVersion(policyId, versionId, data),
    onSuccess: () => { invalidate(); toast.success(t("common.success")); },
  });

  const publishVersion = useMutation({
    mutationFn: ({ expectedRowVersion, versionId }: { expectedRowVersion: number; versionId: string }) =>
      publishOvertimePolicyVersion(policyId, versionId, expectedRowVersion),
    onSuccess: () => { invalidate(); toast.success(t("common.success")); },
  });

  return { createVersion, patchVersion, publishVersion };
}
