import { useMutation, useQueryClient } from "@tanstack/react-query";

import {
  createConditionType,
  updateConditionType,
  deleteConditionType,
  type CreateConditionTypeRequest,
  type UpdateConditionTypeRequest,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";


export function useConditionTypeActions() {
  const queryClient = useQueryClient();


  const createMutation = useMutation({
    mutationFn: (req: CreateConditionTypeRequest) => {

      return createConditionType(req);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("condition-types") });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, req }: { id: string; req: UpdateConditionTypeRequest }) => {

      return updateConditionType(id, req);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("condition-types") });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {

      return deleteConditionType(id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("condition-types") });
    },
  });

  return { createMutation, updateMutation, deleteMutation };
}
