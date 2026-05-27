import { apiClient } from "./client";
import { Employee, EmploymentPeriod, EmployeeDepartmentHistory, WorkingCondition, PaginatedResponse, EmploymentPhaseDefinition, EmployeePhase } from "./types";

export async function fetchEmployees(
  wsId: string,
  page = 1,
  limit = 50
): Promise<PaginatedResponse<Employee>> {
  const { data } = await apiClient.get<PaginatedResponse<Employee>>(
    `/workspaces/${wsId}/employees?page=${page.toString()}&limit=${limit.toString()}`
  );
  return data;
}

// --- Employment Phases API ---

export async function fetchEmploymentPhases(wsId: string): Promise<EmploymentPhaseDefinition[]> {
  const { data } = await apiClient.get<EmploymentPhaseDefinition[]>(`/workspaces/${wsId}/employment-phases`);
  return data;
}

export async function createEmploymentPhase(wsId: string, data: Partial<EmploymentPhaseDefinition>): Promise<EmploymentPhaseDefinition> {
  const { data: result } = await apiClient.post<EmploymentPhaseDefinition>(`/workspaces/${wsId}/employment-phases`, data);
  return result;
}

export async function fetchEmployeePhases(wsId: string, employeeId: string): Promise<EmployeePhase[]> {
  const { data } = await apiClient.get<EmployeePhase[]>(`/workspaces/${wsId}/employees/${employeeId}/phases`);
  return data;
}

export async function assignEmployeePhase(
  wsId: string,
  employeeId: string,
  data: { phase_definition_id: string; start_date: string; end_date: string; override_config?: unknown }
): Promise<EmployeePhase> {
  const { data: result } = await apiClient.post<EmployeePhase>(`/workspaces/${wsId}/employees/${employeeId}/phases`, data);
  return result;
}

export async function updateEmployeePhaseStatus(
  wsId: string,
  employeeId: string,
  phaseId: string,
  data: { status: string; evaluation_notes?: string }
): Promise<EmployeePhase> {
  const { data: result } = await apiClient.put<EmployeePhase>(`/workspaces/${wsId}/employees/${employeeId}/phases/${phaseId}/status`, data);
  return result;
}

export async function fetchWorkingConditions(wsId: string, employeeId: string): Promise<WorkingCondition[]> {
  const { data } = await apiClient.get<WorkingCondition[]>(`/workspaces/${wsId}/employees/${employeeId}/conditions`);
  return data;
}

export async function createWorkingCondition(
  wsId: string,
  employeeId: string,
  condition: Partial<WorkingCondition>
): Promise<WorkingCondition> {
  const { data } = await apiClient.post<WorkingCondition>(`/workspaces/${wsId}/employees/${employeeId}/conditions`, condition);
  return data;
}

export async function deleteWorkingCondition(wsId: string, employeeId: string, conditionId: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/employees/${employeeId}/conditions/${conditionId}`);
}

export interface SearchEmployeesParams {
  departmentIds?: string[];
  limit?: number;
  page?: number;
  query: string;
  sortBy?: string;
  sortDir?: string;
  wsId: string;
}

export async function searchEmployees(
  params: SearchEmployeesParams
): Promise<PaginatedResponse<Employee>> {
  const {
    wsId,
    query,
    page = 1,
    limit = 50,
    sortBy,
    sortDir,
    departmentIds,
  } = params;
  const queryParams = new URLSearchParams();
  if (query) {
    queryParams.append("q", query);
  }
  queryParams.append("page", page.toString());
  queryParams.append("limit", limit.toString());
  if (sortBy) {
    queryParams.append("sort_by", sortBy);
  }
  if (sortDir) {
    queryParams.append("sort_dir", sortDir);
  }
  if (departmentIds && departmentIds.length > 0) {
    queryParams.append("department_id", departmentIds.join(","));
  }
  
  const { data } = await apiClient.get<PaginatedResponse<Employee>>(`/workspaces/${wsId}/employees?${queryParams.toString()}`);
  return data;
}

export async function fetchEmployee(wsId: string, id: string): Promise<Employee> {
  const { data } = await apiClient.get<Employee>(`/workspaces/${wsId}/employees/${id}`);
  return data;
}

export async function fetchEmploymentHistory(
  wsId: string,
  id: string
): Promise<EmploymentPeriod[]> {
  const { data } = await apiClient.get<EmploymentPeriod[]>(`/workspaces/${wsId}/employees/${id}/history`);
  return data;
}

export async function rehireEmployee(
  wsId: string,
  id: string,
  data: { hire_date: string }
): Promise<Employee> {
  const { data: result } = await apiClient.post<Employee>(`/workspaces/${wsId}/employees/${id}/rehire`, data);
  return result;
}

export async function createEmployee(
  wsId: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const { data } = await apiClient.post<Employee>(`/workspaces/${wsId}/employees`, employee);
  return data;
}

export async function updateEmployee(
  wsId: string,
  id: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const { data } = await apiClient.put<Employee>(`/workspaces/${wsId}/employees/${id}`, employee);
  return data;
}

export async function deleteEmployee(wsId: string, id: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/employees/${id}`);
}

export async function transferEmployee(
  wsId: string,
  employeeId: string,
  data: { department_id: string; effective_date: string; notes?: string }
): Promise<EmployeeDepartmentHistory> {
  const { data: result } = await apiClient.post<EmployeeDepartmentHistory>(`/workspaces/${wsId}/employees/${employeeId}/departments`, data);
  return result;
}

export async function fetchEmployeeDepartmentHistory(wsId: string, employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const { data } = await apiClient.get<EmployeeDepartmentHistory[]>(`/workspaces/${wsId}/employees/${employeeId}/departments`);
  return data;
}