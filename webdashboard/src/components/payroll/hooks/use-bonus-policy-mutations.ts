import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { createIncentivePolicy, deleteIncentivePolicy, cloneIncentivePolicy } from "@/lib/api/payroll";

export function useBonusPolicyMutations(wsId: string, onCreateSuccess?: () => void) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createIncentivePolicy>[1]) => createIncentivePolicy(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      onCreateSuccess?.();
      toast.success(t("common.success"));
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteIncentivePolicy(wsId, id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      toast.success(t("common.deleted"));
    },
  });

  const cloneMutation = useMutation({
    mutationFn: (id: string) => cloneIncentivePolicy(wsId, id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["incentive-policies"] });
      toast.success(t("common.success", { defaultValue: "Cloned successfully" }));
    },
  });

  return { createMutation, deleteMutation, cloneMutation };
}
