import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import type { BehalfAssignment } from "@/lib/api/types";

import {
  fetchBehalfAssignments,
  createBehalfAssignment,
  deleteBehalfAssignment,
  fetchLeaveBehalfAssignments,
  createLeaveBehalfAssignment,
  deleteLeaveBehalfAssignment,
  fetchAdjustmentBehalfAssignments,
  createAdjustmentBehalfAssignment,
  deleteAdjustmentBehalfAssignment,
  overtimeRequestsApi,
} from "@/lib/api";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";

export type BehalfAssignmentType = "ot" | "leave" | "attendance_adjustment" | "overtime_request";

export const behalfKeys = {
  all: ["behalf-assignments"] as const,
  list: (wsId: string, type: BehalfAssignmentType) => [...behalfKeys.all, "list", wsId, type] as const,
};

export function useBehalfAssignments(type: BehalfAssignmentType = "ot") {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();

  const listQuery = useQuery({
    queryKey: wsId ? behalfKeys.list(wsId, type) : [],
    queryFn: async (): Promise<BehalfAssignment[]> => {
      if (!wsId) throw new Error("No workspace");
      switch (type) {
        case "ot":
          return fetchBehalfAssignments(wsId);
        case "leave":
          return fetchLeaveBehalfAssignments(wsId);
        case "attendance_adjustment":
          return fetchAdjustmentBehalfAssignments(wsId);
        case "overtime_request":
          return overtimeRequestsApi.listRequestBehalfAssignments(wsId);
      }
    },
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: {
      assignee_user_id: string;
      target_department_id?: string | null;
      target_employee_id?: string | null;
    }) => {
      if (!wsId) throw new Error("No workspace");
      switch (type) {
        case "ot":
          return createBehalfAssignment(wsId, data);
        case "leave":
          return createLeaveBehalfAssignment(wsId, data);
        case "attendance_adjustment":
          return createAdjustmentBehalfAssignment(wsId, data);
        case "overtime_request":
          return overtimeRequestsApi.createRequestBehalfAssignment(wsId, data);
      }
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: behalfKeys.list(wsId, type) });
      }
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (!wsId) throw new Error("No workspace");
      switch (type) {
        case "ot":
          return deleteBehalfAssignment(wsId, id);
        case "leave":
          return deleteLeaveBehalfAssignment(wsId, id);
        case "attendance_adjustment":
          return deleteAdjustmentBehalfAssignment(wsId, id);
        case "overtime_request":
          return overtimeRequestsApi.deleteRequestBehalfAssignment(wsId, id);
      }
    },
    onSuccess: () => {
      if (wsId) {
        void queryClient.invalidateQueries({ queryKey: behalfKeys.list(wsId, type) });
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
