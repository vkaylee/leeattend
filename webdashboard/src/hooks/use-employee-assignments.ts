import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type {
  CreateEmployeeAssignmentRequest,
  UpdateEmployeeAssignmentRequest,
} from "@/lib/api/types";

import {
  createEmployeeAssignment,
  deleteEmployeeAssignment,
  fetchEmployeeAssignments,
  fetchWorkspaceAssignments,
  updateEmployeeAssignment,
  type EmployeeAssignmentWithEmployee,
} from "@/lib/api/employee-assignments";

export const employeeAssignmentKeys = {
  all: ["employee-assignments"] as const,
  list: (employeeId: string) =>
    [...employeeAssignmentKeys.all, employeeId] as const,
  workspace: () =>
    [...employeeAssignmentKeys.all, "workspace"] as const,
};
export const useAssignments = useWorkspaceAssignments;
export function useWorkspaceAssignments() {
  return useQuery<EmployeeAssignmentWithEmployee[]>({
    queryKey: employeeAssignmentKeys.workspace(),
    queryFn: () => fetchWorkspaceAssignments(),
  });
}

export function useEmployeeAssignments(employeeId: string | null | undefined) {
  return useQuery({
    queryKey: employeeId ? employeeAssignmentKeys.list(employeeId) : [],
    queryFn: () => {
      if (!employeeId) throw new Error("Missing params");
      return fetchEmployeeAssignments(employeeId);
    },
    enabled: !!employeeId,
  });
}

export function useCreateEmployeeAssignment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      employeeId,
      data,
    }: {
      employeeId: string;
      data: CreateEmployeeAssignmentRequest;
    }) => createEmployeeAssignment(employeeId, data),
    onSuccess: (data, variables) => {
      void queryClient.invalidateQueries({
        queryKey: employeeAssignmentKeys.all,
      });
    },
  });
}

export function useUpdateEmployeeAssignment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      employeeId,
      assignmentId,
      data,
    }: {
      employeeId: string;
      assignmentId: string;
      data: UpdateEmployeeAssignmentRequest;
    }) => updateEmployeeAssignment(employeeId, assignmentId, data),
    onSuccess: (data, variables) => {
      void queryClient.invalidateQueries({
        queryKey: employeeAssignmentKeys.all,
      });
    },
  });
}

export function useDeleteEmployeeAssignment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      employeeId,
      assignmentId,
    }: {
      employeeId: string;
      assignmentId: string;
    }) => deleteEmployeeAssignment(employeeId, assignmentId),
    onSuccess: (_, variables) => {
      void queryClient.invalidateQueries({
        queryKey: employeeAssignmentKeys.all,
      });
    },
  });
}
