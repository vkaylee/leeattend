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
  employeeId: string,
): Promise<EmployeeAssignment[]> => {
  const { data } = await client.GET<EmployeeAssignment[]>("/v1/employees/{employee_id}/assignments", {
    params: {
      path: { employee_id: employeeId },
    },
  });
  return data;
};

export const createEmployeeAssignment = async (
  employeeId: string,
  data: CreateEmployeeAssignmentRequest,
): Promise<EmployeeAssignment> => {
  const { data: result } = await client.POST<EmployeeAssignment>("/v1/employees/{employee_id}/assignments", {
    params: {
      path: { employee_id: employeeId },
    },
    body: data,
  });
  return result;
};

export const updateEmployeeAssignment = async (
  employeeId: string,
  assignmentId: string,
  data: UpdateEmployeeAssignmentRequest,
): Promise<EmployeeAssignment> => {
  const { data: result } = await client.PATCH<EmployeeAssignment>("/v1/employees/{employee_id}/assignments/{assignment_id}", {
    params: {
      path: { employee_id: employeeId, assignment_id: assignmentId },
    },
    body: data,
  });
  return result;
};

export const deleteEmployeeAssignment = async (
  employeeId: string,
  assignmentId: string,
): Promise<void> => {
  await client.DELETE("/v1/employees/{employee_id}/assignments/{assignment_id}", {
    params: {
      path: { employee_id: employeeId, assignment_id: assignmentId },
    },
  });
};

export const fetchWorkspaceAssignments = async (): Promise<EmployeeAssignmentWithEmployee[]> => {
  const { data } = await client.GET<EmployeeAssignmentWithEmployee[]>("/v1/assignments");
  return data;
};
