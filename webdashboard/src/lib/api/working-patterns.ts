import { client } from "./client";
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

export type {
  AssignPatternRequest,
  BulkAssignPatternRequest,
  BulkAssignResponse,
  CreateWorkingPatternRequest,
  EmployeeWorkingPattern,
  UpdateWorkingPatternRequest,
  WorkingPattern, 
  WorkingPatternDetail, 
};

export async function fetchWorkingPatterns(wsId: string): Promise<WorkingPattern[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/working-patterns", {
    params: {
      path: { workspace_id: wsId },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as WorkingPattern[];
}

export async function createWorkingPattern(wsId: string, data: CreateWorkingPatternRequest): Promise<WorkingPattern> {
  const { data: result } = await client.POST("/workspaces/{workspace_id}/working-patterns", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result as unknown as WorkingPattern;
}

export async function updateWorkingPattern(
  wsId: string,
  patternId: string,
  data: UpdateWorkingPatternRequest
): Promise<WorkingPattern> {
  const { data: result } = await client.PUT("/workspaces/{workspace_id}/working-patterns/{pattern_id}", {
    params: {
      path: { workspace_id: wsId, pattern_id: patternId },
    },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result as unknown as WorkingPattern;
}

export async function fetchPatternDetails(wsId: string, patternId: string): Promise<WorkingPatternDetail[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/working-patterns/{pattern_id}", {
    params: {
      path: { workspace_id: wsId, pattern_id: patternId },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as WorkingPatternDetail[];
}

export async function assignPattern(
  wsId: string, 
  employeeId: string, 
  data: AssignPatternRequest
): Promise<EmployeeWorkingPattern> {
  const { data: result } = await client.POST("/workspaces/{workspace_id}/employees/{id}/working-patterns", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result as unknown as EmployeeWorkingPattern;
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
  return result as unknown as BulkAssignResponse;
}

export async function fetchEmployeePatternHistory(
  wsId: string, 
  employeeId: string
): Promise<EmployeeWorkingPattern[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/working-patterns", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as EmployeeWorkingPattern[];
}

export async function deleteWorkingPattern(wsId: string, patternId: string): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/working-patterns/{pattern_id}", {
    params: {
      path: { workspace_id: wsId, pattern_id: patternId },
    },
  });
}
