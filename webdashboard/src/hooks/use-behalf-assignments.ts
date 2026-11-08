
import { useBehalfAssignmentData } from "./use-behalf-assignment-data";
import { useBehalfAssignmentMutations } from "./use-behalf-assignment-mutations";

export type BehalfAssignmentType = "ot" | "leave" | "attendance_adjustment" | "overtime_request";

export const behalfKeys = {
  all: ["behalf-assignments"] as const,
  list: (type: BehalfAssignmentType) => [...behalfKeys.all, "list", type] as const,
};

export function useBehalfAssignments(type: BehalfAssignmentType = "ot") {
  const { assignments, isLoading, isError, refetch } = useBehalfAssignmentData(type);
  const { createAssignment, isCreating, deleteAssignment, isDeleting } = useBehalfAssignmentMutations(type);

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
