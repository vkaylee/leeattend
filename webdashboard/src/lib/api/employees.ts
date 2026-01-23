import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Employee, EmploymentPeriod, EmployeeDepartmentHistory, WorkingCondition, PaginatedResponse } from "./types";

export async function fetchEmployees(
  wsId: string,
  page = 1,
  limit = 50
): Promise<PaginatedResponse<Employee>> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/employees?page=${page}&limit=${limit}`
  );
  if (!response.ok) throw new Error("Failed to fetch employees");
  return response.json();
}

export async function fetchWorkingConditions(wsId: string, employeeId: string): Promise<WorkingCondition[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/conditions`);
  if (!response.ok) throw new Error("Failed to fetch working conditions");
  return response.json();
}

export async function createWorkingCondition(
  wsId: string,
  employeeId: string,
  condition: Partial<WorkingCondition>
): Promise<WorkingCondition> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/conditions`, {
    method: "POST",
    body: JSON.stringify(condition),
  });
  if (!response.ok) throw new Error("Failed to create working condition");
  return response.json();
}

export async function deleteWorkingCondition(wsId: string, employeeId: string, conditionId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/conditions/${conditionId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete working condition");
}

export async function searchEmployees(
  wsId: string, 
  query: string,
  page = 1,
  limit = 50
): Promise<PaginatedResponse<Employee>> {
  const queryParams = new URLSearchParams();
  if (query) {
    queryParams.append("q", query);
  }
  queryParams.append("page", page.toString());
  queryParams.append("limit", limit.toString());
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees?${queryParams.toString()}`);
  if (!response.ok) throw new Error("Failed to search employees");
  return response.json();
}

export async function fetchEmployee(wsId: string, id: string): Promise<Employee> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}`);
  if (!response.ok) throw new Error("Failed to fetch employee");
  return response.json();
}

export async function fetchEmploymentHistory(
  wsId: string,
  id: string
): Promise<EmploymentPeriod[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}/history`);
  if (!response.ok) throw new Error("Failed to fetch employment history");
  return response.json();
}

export async function rehireEmployee(
  wsId: string,
  id: string,
  data: { hire_date: string }
): Promise<Employee> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}/rehire`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to re-hire employee");
  }
  return response.json();
}

export async function createEmployee(
  wsId: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees`, {
    method: "POST",
    body: JSON.stringify(employee),
  });
  if (!response.ok) throw new Error("Failed to create employee");
  return response.json();
}

export async function updateEmployee(
  wsId: string,
  id: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}`, {
    method: "PUT",
    body: JSON.stringify(employee),
  });
  if (!response.ok) throw new Error("Failed to update employee");
  return response.json();
}

export async function deleteEmployee(wsId: string, id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete employee");
}

export async function transferEmployee(
  wsId: string,
  employeeId: string,
  data: { department_id: string; effective_date: string; notes?: string }
): Promise<EmployeeDepartmentHistory> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/departments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to transfer employee");
  return response.json();
}

export async function fetchEmployeeDepartmentHistory(wsId: string, employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/departments`);
  if (!response.ok) throw new Error("Failed to fetch dept history");
  return response.json();
}