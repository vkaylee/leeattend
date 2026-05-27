import { apiClient } from "./client";
import { 
  AssignPatternRequest,
  BulkAssignPatternRequest,
  BulkAssignResponse,
  CreateWorkingPatternRequest,
  EmployeeWorkingPattern,
  UpdateWorkingPatternRequest,
  WorkingPattern, 
  WorkingPatternDetail, 
} from "./types";

export async function fetchWorkingPatterns(wsId: string): Promise<WorkingPattern[]> {
  const { data } = await apiClient.get<WorkingPattern[]>(`/workspaces/${wsId}/working-patterns`);
  return data;
}

export async function createWorkingPattern(wsId: string, data: CreateWorkingPatternRequest): Promise<WorkingPattern> {
  const { data: result } = await apiClient.post<WorkingPattern>(`/workspaces/${wsId}/working-patterns`, data);
  return result;
}

export async function updateWorkingPattern(
  wsId: string,
  patternId: string,
  data: UpdateWorkingPatternRequest
): Promise<WorkingPattern> {
  const { data: result } = await apiClient.put<WorkingPattern>(`/workspaces/${wsId}/working-patterns/${patternId}`, data);
  return result;
}

export async function fetchPatternDetails(wsId: string, patternId: string): Promise<WorkingPatternDetail[]> {
  const { data } = await apiClient.get<WorkingPatternDetail[]>(`/workspaces/${wsId}/working-patterns/${patternId}`);
  return data;
}

export async function assignPattern(
  wsId: string, 
  employeeId: string, 
  data: AssignPatternRequest
): Promise<EmployeeWorkingPattern> {
  const { data: result } = await apiClient.post<EmployeeWorkingPattern>(`/workspaces/${wsId}/employees/${employeeId}/working-patterns`, data);
  return result;
}

export async function bulkAssignPattern(
  wsId: string,
  data: BulkAssignPatternRequest
): Promise<BulkAssignResponse> {
  const { data: result } = await apiClient.post<BulkAssignResponse>(`/workspaces/${wsId}/working-patterns/bulk-assign`, data);
  return result;
}

export async function fetchEmployeePatternHistory(
  wsId: string, 
  employeeId: string
): Promise<EmployeeWorkingPattern[]> {
  const { data } = await apiClient.get<EmployeeWorkingPattern[]>(`/workspaces/${wsId}/employees/${employeeId}/working-patterns`);
  return data;
}

export async function deleteWorkingPattern(wsId: string, patternId: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/working-patterns/${patternId}`);
}
