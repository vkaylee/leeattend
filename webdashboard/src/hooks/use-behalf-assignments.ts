import { getCurrentWorkspaceId } from "@/lib/api/helpers";

import { useBehalfAssignmentData } from "./use-behalf-assignment-data";
import { useBehalfAssignmentMutations } from "./use-behalf-assignment-mutations";

export type BehalfAssignmentType = "ot" | "leave" | "attendance_adjustment" | "overtime_request";

export const behalfKeys = {
  all: ["behalf-assignments"] as const,
  list: (wsId: string, type: BehalfAssignmentType) => [...behalfKeys.all, "list", wsId, type] as const,
};

export function useBehalfAssignments(type: BehalfAssignmentType = "ot") {
  const wsId = getCurrentWorkspaceId();

  const { assignments, isLoading, isError, refetch } = useBehalfAssignmentData(type, wsId);
  const { createAssignment, isCreating, deleteAssignment, isDeleting } = useBehalfAssignmentMutations(type, wsId);

  return {
    assignments,
    isLoading,
    isError,
    refetch,
    createAssignment,
    isCreating,
    deleteAssignment,
    isDeleting,
  };
}
