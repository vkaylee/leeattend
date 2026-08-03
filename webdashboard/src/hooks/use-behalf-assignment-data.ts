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

export function useBehalfAssignmentData(
  type: BehalfAssignmentType,
  wsId: string | null,
) {
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

  return {
    assignments: listQuery.data ?? [],
    isLoading: listQuery.isLoading,
    isError: listQuery.isError,
    refetch: listQuery.refetch,
  };
}
