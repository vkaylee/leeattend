import { useQuery } from "@tanstack/react-query";

import type { BehalfAssignment } from "@/lib/api/types";

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
          return fetchBehalfAssignments();
        case "leave":
          return fetchLeaveBehalfAssignments();
        case "attendance_adjustment":
          return fetchAdjustmentBehalfAssignments();
        case "overtime_request":
          return overtimeRequestsApi.listRequestBehalfAssignments();
      }
    },
  });

  return {
    assignments: listQuery.data ?? [],
    isLoading: listQuery.isLoading,
    isError: listQuery.isError,
    refetch: listQuery.refetch,
  };
}
