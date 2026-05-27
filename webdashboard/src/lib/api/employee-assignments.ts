import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import type {
  CreateEmployeeAssignmentRequest,
  EmployeeAssignment,
  UpdateEmployeeAssignmentRequest,
} from "./types";

export const fetchEmployeeAssignments = async (
  workspaceId: string,
  employeeId: string,
): Promise<EmployeeAssignment[]> => {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${workspaceId}/employees/${employeeId}/assignments`,
  );
  if (!response.ok) throw new Error("Failed to fetch employee assignments");
  return response.json() as Promise<EmployeeAssignment[]>;
};

export const createEmployeeAssignment = async (
  workspaceId: string,
  employeeId: string,
  data: CreateEmployeeAssignmentRequest,
): Promise<EmployeeAssignment> => {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${workspaceId}/employees/${employeeId}/assignments`,
    {
      method: "POST",
      body: JSON.stringify(data),
    },
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to create employee assignment");
  }
  return response.json() as Promise<EmployeeAssignment>;
};

export const updateEmployeeAssignment = async (
  workspaceId: string,
  employeeId: string,
  assignmentId: string,
  data: UpdateEmployeeAssignmentRequest,
): Promise<EmployeeAssignment> => {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${workspaceId}/employees/${employeeId}/assignments/${assignmentId}`,
    {
      method: "PATCH",
      body: JSON.stringify(data),
    },
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to update employee assignment");
  }
  return response.json() as Promise<EmployeeAssignment>;
};

export const deleteEmployeeAssignment = async (
  workspaceId: string,
  employeeId: string,
  assignmentId: string,
): Promise<void> => {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${workspaceId}/employees/${employeeId}/assignments/${assignmentId}`,
    {
      method: "DELETE",
    },
  );
  if (!response.ok) throw new Error("Failed to delete employee assignment");
};
