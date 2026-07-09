import type { 
  AssignPatternRequest,
  BulkAssignPatternRequest,
  BulkAssignResponse,
  CreateWorkingPatternRequest,
  EmployeeWorkingPattern,
  UpdateWorkingPatternRequest,
  WorkingPattern, 
  WorkingPatternDetail, 
} from "./types";

import { client, fetch } from "./client";



export async function fetchWorkingPatterns(wsId: string): Promise<WorkingPattern[]> {
  const { data } = await fetch<WorkingPattern[]>("GET", "/workspaces/{workspace_id}/working-patterns", {
    params: { path: { workspace_id: wsId } },
  });
  if (!data) throw new Error("No response data");
  return data;
}

export async function createWorkingPattern(wsId: string, data: CreateWorkingPatternRequest): Promise<WorkingPattern> {
  const { data: result } = await fetch<WorkingPattern>("POST", "/workspaces/{workspace_id}/working-patterns", {
    params: { path: { workspace_id: wsId } },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result;
}

export async function updateWorkingPattern(
  wsId: string,
  patternId: string,
  data: UpdateWorkingPatternRequest
): Promise<WorkingPattern> {
  const { data: result } = await fetch<WorkingPattern>("PUT", "/workspaces/{workspace_id}/working-patterns/{pattern_id}", {
    params: { path: { workspace_id: wsId, pattern_id: patternId } },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result;
}

export async function fetchPatternDetails(wsId: string, patternId: string): Promise<WorkingPatternDetail[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/working-patterns/{pattern_id}", {
    params: {
      path: { workspace_id: wsId, pattern_id: patternId },
    },
  });
  if (!data) throw new Error("No response data");
  return data;
}

export async function assignPattern(
  wsId: string, 
  employmentId: string, 
  data: AssignPatternRequest
): Promise<EmployeeWorkingPattern> {
  const { data: result } = await client.POST("/workspaces/{workspace_id}/employments/{id}/working-patterns", {
    params: {
      path: { workspace_id: wsId, id: employmentId },
    },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result;
}

export async function bulkAssignPattern(
  wsId: string,
  data: BulkAssignPatternRequest
): Promise<BulkAssignResponse> {
  const { data: result } = await client.POST("/workspaces/{workspace_id}/working-patterns/bulk-assign", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result;
}

export async function fetchEmployeePatternHistory(
  wsId: string, 
  employmentId: string
): Promise<EmployeeWorkingPattern[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employments/{id}/working-patterns", {
    params: {
      path: { workspace_id: wsId, id: employmentId },
    },
  });
  if (!data) throw new Error("No response data");
  return data;
}

export async function deleteWorkingPattern(wsId: string, patternId: string): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/working-patterns/{pattern_id}", {
    params: {
      path: { workspace_id: wsId, pattern_id: patternId },
    },
  });
}
