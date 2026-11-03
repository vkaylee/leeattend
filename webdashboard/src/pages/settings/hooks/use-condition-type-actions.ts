import { useMutation, useQueryClient } from "@tanstack/react-query";

import {
  createConditionType,
  updateConditionType,
  deleteConditionType,
  getStandaloneOrganizationId,
  type CreateConditionTypeRequest,
  type UpdateConditionTypeRequest,
} from "@/lib/api";

export function useConditionTypeActions() {
  const queryClient = useQueryClient();
  const wsId = getStandaloneOrganizationId();

  const createMutation = useMutation({
    mutationFn: (req: CreateConditionTypeRequest) => {
      if (!wsId) throw new Error("No workspace");
      return createConditionType(wsId, req);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["condition-types", wsId] });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, req }: { id: string; req: UpdateConditionTypeRequest }) => {
      if (!wsId) throw new Error("No workspace");
      return updateConditionType(wsId, id, req);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["condition-types", wsId] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (!wsId) throw new Error("No workspace");
      return deleteConditionType(wsId, id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["condition-types", wsId] });
    },
  });

  return { createMutation, updateMutation, deleteMutation };
}
