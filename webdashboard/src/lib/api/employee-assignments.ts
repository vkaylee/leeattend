import { client } from "./client";
import type {
  CreateEmployeeAssignmentRequest,
  EmployeeAssignment,
  UpdateEmployeeAssignmentRequest,
} from "./types";

export type {
  CreateEmployeeAssignmentRequest,
  EmployeeAssignment,
  UpdateEmployeeAssignmentRequest,
};

export const fetchEmployeeAssignments = async (
  workspaceId: string,
  employeeId: string,
): Promise<EmployeeAssignment[]> => {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employees/{employee_id}/assignments", {
    params: {
      path: { ws_id: workspaceId, employee_id: employeeId },
    },
  });
  if (!data) throw new Error("Failed to fetch employee assignments");
  return data as unknown as EmployeeAssignment[];
};

export const createEmployeeAssignment = async (
  workspaceId: string,
  employeeId: string,
  data: CreateEmployeeAssignmentRequest,
): Promise<EmployeeAssignment> => {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employees/{employee_id}/assignments", {
    params: {
      path: { ws_id: workspaceId, employee_id: employeeId },
    },
    body: data as any,
  });
  if (!result) throw new Error("Failed to create employee assignment");
  return result as unknown as EmployeeAssignment;
};

export const updateEmployeeAssignment = async (
  workspaceId: string,
  employeeId: string,
  assignmentId: string,
  data: UpdateEmployeeAssignmentRequest,
): Promise<EmployeeAssignment> => {
  const { data: result } = await client.PATCH("/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", {
    params: {
      path: { ws_id: workspaceId, employee_id: employeeId, assignment_id: assignmentId },
    },
    body: data as any,
  });
  if (!result) throw new Error("Failed to update employee assignment");
  return result as unknown as EmployeeAssignment;
};

export const deleteEmployeeAssignment = async (
  workspaceId: string,
  employeeId: string,
  assignmentId: string,
): Promise<void> => {
  await client.DELETE("/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", {
    params: {
      path: { ws_id: workspaceId, employee_id: employeeId, assignment_id: assignmentId },
    },
  });
};
