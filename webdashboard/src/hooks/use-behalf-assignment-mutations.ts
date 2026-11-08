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

export function useBehalfAssignmentMutations(type: BehalfAssignmentType) {
  const queryClient = useQueryClient();

  const createMutation = useMutation({
    mutationFn: (data: {
      assignee_user_id: string;
      target_department_id?: string | null;
      target_employee_id?: string | null;
    }) => {
      switch (type) {
        case "ot":
          return createBehalfAssignment(data);
        case "leave":
          return createLeaveBehalfAssignment(data);
        case "attendance_adjustment":
          return createAdjustmentBehalfAssignment(data);
        case "overtime_request":
          return overtimeRequestsApi.createRequestBehalfAssignment(data);
      }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: behalfKeys.list(type) });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      switch (type) {
        case "ot":
          return deleteBehalfAssignment(id);
        case "leave":
          return deleteLeaveBehalfAssignment(id);
        case "attendance_adjustment":
          return deleteAdjustmentBehalfAssignment(id);
        case "overtime_request":
          return overtimeRequestsApi.deleteRequestBehalfAssignment(id);
      }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: behalfKeys.list(type) });
    },
  });

  return {
    createAssignment: createMutation.mutateAsync,
    isCreating: createMutation.isPending,
    deleteAssignment: deleteMutation.mutateAsync,
    isDeleting: deleteMutation.isPending,
  };
}
