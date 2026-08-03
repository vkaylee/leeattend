import { useMutation, useQueryClient } from "@tanstack/react-query";

import {
  createBehalfAssignment,
  deleteBehalfAssignment,
  createLeaveBehalfAssignment,
  deleteLeaveBehalfAssignment,
  createAdjustmentBehalfAssignment,
  deleteAdjustmentBehalfAssignment,
  overtimeRequestsApi,
} from "@/lib/api";

import type { BehalfAssignmentType } from "./use-behalf-assignments";

import { behalfKeys } from "./use-behalf-assignments";

export function useBehalfAssignmentMutations(
  type: BehalfAssignmentType,
  wsId: string | null,
) {
  const queryClient = useQueryClient();

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
    createAssignment: createMutation.mutateAsync,
    isCreating: createMutation.isPending,
    deleteAssignment: deleteMutation.mutateAsync,
    isDeleting: deleteMutation.isPending,
  };
}
