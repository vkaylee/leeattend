import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { 
  WorkingPattern, 
  WorkingPatternDetail, 
  EmployeeWorkingPattern,
  CreateWorkingPatternRequest,
  AssignPatternRequest,
  BulkAssignPatternRequest,
  BulkAssignResponse
} from "./types";

export async function fetchWorkingPatterns(wsId: string): Promise<WorkingPattern[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/working-patterns`);
  if (!response.ok) throw new Error("Failed to fetch working patterns");
  return response.json();
}

export async function createWorkingPattern(wsId: string, data: CreateWorkingPatternRequest): Promise<WorkingPattern> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/working-patterns`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create working pattern");
  return response.json();
}

export async function fetchPatternDetails(wsId: string, patternId: string): Promise<WorkingPatternDetail[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/working-patterns/${patternId}`);
  if (!response.ok) throw new Error("Failed to fetch pattern details");
  return response.json();
}

export async function assignPattern(
  wsId: string, 
  employeeId: string, 
  data: AssignPatternRequest
): Promise<EmployeeWorkingPattern> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/working-patterns`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to assign pattern");
  return response.json();
}

export async function bulkAssignPattern(
  wsId: string,
  data: BulkAssignPatternRequest
): Promise<BulkAssignResponse> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/working-patterns/bulk-assign`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to bulk assign pattern");
  return response.json();
}

export async function fetchEmployeePatternHistory(
  wsId: string, 
  employeeId: string
): Promise<EmployeeWorkingPattern[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/working-patterns`);
  if (!response.ok) throw new Error("Failed to fetch pattern history");
  return response.json();
}

export async function deleteWorkingPattern(wsId: string, patternId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/working-patterns/${patternId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete working pattern");
}
