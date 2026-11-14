import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { deleteAssignmentRule } from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";

import { useAssignmentRulesData } from "./use-assignment-rules-data";


export function useAssignmentRules() {
  const { t } = useTranslation();
  const { rules, rulesLoading, patterns, departments, employers, locations } = useAssignmentRulesData();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { confirm, ConfirmationDialog } = useConfirm();

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteAssignmentRule(id),
    onSuccess: () => {
      toast.success(t("rules.messages.delete_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("assignment-rules") });
    },
  });

  const handleDelete = async (id: string) => {
    const ok = await confirm({
      title: t("common.confirm_delete"),
      message: t("rules.confirm_delete_msg", "Are you sure you want to delete this rule?"),
      variant: "danger",
    });
    if (ok) {
      deleteMutation.mutate(id);
    }
  };

  return {
    t,
    rules,
    rulesLoading,
    patterns,
    departments,
    employers,
    locations,
    handleDelete,
    ConfirmationDialog,
  };
}
