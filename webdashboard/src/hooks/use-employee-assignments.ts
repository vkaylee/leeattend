import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type {
  CreateEmployeeAssignmentRequest,
  UpdateEmployeeAssignmentRequest,
} from "@/lib/api/types";

import {
  createEmployeeAssignment,
  deleteEmployeeAssignment,
  fetchEmployeeAssignments,
  updateEmployeeAssignment,
} from "@/lib/api/employee-assignments";

export const employeeAssignmentKeys = {
  all: ["employee-assignments"] as const,
  list: (wsId: string, employeeId: string) =>
    [...employeeAssignmentKeys.all, wsId, employeeId] as const,
};

export function useEmployeeAssignments(wsId: string | null | undefined, employeeId: string | null | undefined) {
  return useQuery({
    queryKey: wsId && employeeId ? employeeAssignmentKeys.list(wsId, employeeId) : [],
    queryFn: () => {
      if (!wsId || !employeeId) throw new Error("Missing params");
      return fetchEmployeeAssignments(wsId, employeeId);
    },
    enabled: !!wsId && !!employeeId,
  });
}

export function useCreateEmployeeAssignment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      wsId,
      employeeId,
      data,
    }: {
      wsId: string;
      employeeId: string;
      data: CreateEmployeeAssignmentRequest;
    }) => createEmployeeAssignment(wsId, employeeId, data),
    onSuccess: (data, variables) => {
      void queryClient.invalidateQueries({
        queryKey: employeeAssignmentKeys.list(variables.wsId, variables.employeeId),
      });
    },
  });
}

export function useUpdateEmployeeAssignment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      wsId,
      employeeId,
      assignmentId,
      data,
    }: {
      wsId: string;
      employeeId: string;
      assignmentId: string;
      data: UpdateEmployeeAssignmentRequest;
    }) => updateEmployeeAssignment(wsId, employeeId, assignmentId, data),
    onSuccess: (data, variables) => {
      void queryClient.invalidateQueries({
        queryKey: employeeAssignmentKeys.list(variables.wsId, variables.employeeId),
      });
    },
  });
}

export function useDeleteEmployeeAssignment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      wsId,
      employeeId,
      assignmentId,
    }: {
      wsId: string;
      employeeId: string;
      assignmentId: string;
    }) => deleteEmployeeAssignment(wsId, employeeId, assignmentId),
    onSuccess: (_, variables) => {
      void queryClient.invalidateQueries({
        queryKey: employeeAssignmentKeys.list(variables.wsId, variables.employeeId),
      });
    },
  });
}
