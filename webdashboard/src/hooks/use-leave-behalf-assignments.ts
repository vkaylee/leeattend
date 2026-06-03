import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchLeaveBehalfAssignments,
  createLeaveBehalfAssignment,
  deleteLeaveBehalfAssignment,
} from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

export const leaveBehalfKeys = {
  all: ["leave-behalf-assignments"] as const,
  list: (wsId: string) => [...leaveBehalfKeys.all, "list", wsId] as const,
};

export function useLeaveBehalfAssignments() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();

  const listQuery = useQuery({
    queryKey: wsId ? leaveBehalfKeys.list(wsId) : [],
    queryFn: () => {
      if (!wsId) throw new Error("No workspace");
      return fetchLeaveBehalfAssignments(wsId);
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
      return createLeaveBehalfAssignment(wsId, data);
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: leaveBehalfKeys.list(wsId) });
      }
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (!wsId) throw new Error("No workspace");
      return deleteLeaveBehalfAssignment(wsId, id);
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: leaveBehalfKeys.list(wsId) });
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
