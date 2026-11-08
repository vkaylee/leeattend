import { useQuery } from "@tanstack/react-query";

import type { BehalfAssignment } from "@/lib/api/types";
import type { OtClaimBehalfAssignment } from "@/lib/api/attendance-monitoring";

import {
  fetchBehalfAssignments,
  fetchLeaveBehalfAssignments,
  fetchAdjustmentBehalfAssignments,
  overtimeRequestsApi,
} from "@/lib/api";

import type { BehalfAssignmentType } from "./use-behalf-assignments";

import { behalfKeys } from "./use-behalf-assignments";

export function useBehalfAssignmentData(type: BehalfAssignmentType) {
  const listQuery = useQuery({
    queryKey: behalfKeys.list(type),
    queryFn: async (): Promise<BehalfAssignment[]> => {
      switch (type) {
        case "ot":
        case "attendance_adjustment": {
          const assignments = type === "ot"
            ? await fetchBehalfAssignments()
            : await fetchAdjustmentBehalfAssignments();
          return assignments.map((assignment: OtClaimBehalfAssignment) => ({
            ...assignment,
            workspace_id: assignment.workspace_id ?? "",
          }));
        }
        case "leave":
          return fetchLeaveBehalfAssignments();
        case "overtime_request":
          return overtimeRequestsApi.listRequestBehalfAssignments();
      }
    }
  });

  return {
    assignments: listQuery.data ?? [],
    isLoading: listQuery.isLoading,
    isError: listQuery.isError,
    refetch: listQuery.refetch,
  };
}
