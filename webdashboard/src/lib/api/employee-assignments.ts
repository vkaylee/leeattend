import { ApiError } from "./errors";
import { apiFetch } from "./helpers";
import type {
  CreateEmployeeAssignmentRequest,
  EmployeeAssignment,
  UpdateEmployeeAssignmentRequest,
} from "./types";

import { client } from "./client";

export interface EmployeeAssignmentWithEmployee extends EmployeeAssignment {
  employee_id: string;
  employee_name: string;
  employee_code?: string;
  home_location_name?: string;
  host_location_name?: string;
}

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
  return data;
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
    body: data,
  });
  return result;
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
    body: data,
  });
  return result;
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

export const fetchWorkspaceAssignments = async (
  workspaceId: string,
): Promise<EmployeeAssignmentWithEmployee[]> => {
  const res = await apiFetch(`/v1/workspaces/${encodeURIComponent(workspaceId)}/assignments`);
  if (!res.ok) await ApiError.rejectResponse(res);
  return (await res.json()) as EmployeeAssignmentWithEmployee[];
};
