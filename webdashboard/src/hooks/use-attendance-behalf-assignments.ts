import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchAdjustmentBehalfAssignments,
  createAdjustmentBehalfAssignment,
  deleteAdjustmentBehalfAssignment,
} from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

export const adjustmentBehalfKeys = {
  all: ["adjustment-behalf-assignments"] as const,
  list: (wsId: string) => [...adjustmentBehalfKeys.all, "list", wsId] as const,
};

export function useAttendanceBehalfAssignments() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();

  const listQuery = useQuery({
    queryKey: wsId ? adjustmentBehalfKeys.list(wsId) : [],
    queryFn: () => {
      if (!wsId) throw new Error("No workspace");
      return fetchAdjustmentBehalfAssignments(wsId);
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
      return createAdjustmentBehalfAssignment(wsId, data);
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: adjustmentBehalfKeys.list(wsId) });
      }
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (!wsId) throw new Error("No workspace");
      return deleteAdjustmentBehalfAssignment(wsId, id);
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: adjustmentBehalfKeys.list(wsId) });
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
