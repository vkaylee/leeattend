import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchBehalfAssignments,
  createBehalfAssignment,
  deleteBehalfAssignment,
} from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

export const behalfKeys = {
  all: ["behalf-assignments"] as const,
  list: (wsId: string) => [...behalfKeys.all, "list", wsId] as const,
};

export function useBehalfAssignments() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();

  const listQuery = useQuery({
    queryKey: wsId ? behalfKeys.list(wsId) : [],
    queryFn: () => {
      if (!wsId) throw new Error("No workspace");
      return fetchBehalfAssignments(wsId);
    },
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: {
      assignee_employee_id: string;
      target_employee_id?: string | null;
      target_department_id?: string | null;
    }) => {
      if (!wsId) throw new Error("No workspace");
      return createBehalfAssignment(wsId, data);
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: behalfKeys.list(wsId) });
      }
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (!wsId) throw new Error("No workspace");
      return deleteBehalfAssignment(wsId, id);
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: behalfKeys.list(wsId) });
      }
    },
  });

  return {
    assignments: listQuery.data ?? [],
    isLoading: listQuery.isLoading,
    isError: listQuery.isError,
    refetch: listQuery.refetch,
    createAssignment: createMutation.mutateAsync,
    isCreating: createMutation.isPending,
    deleteAssignment: deleteMutation.mutateAsync,
    isDeleting: deleteMutation.isPending,
  };
}
