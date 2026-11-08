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

import { client } from "./client";

export async function fetchWorkingPatterns(): Promise<WorkingPattern[]> {
  const { data } = await client.GET<WorkingPattern[]>("/v1/working-patterns");
  return data;
}

export async function createWorkingPattern(data: CreateWorkingPatternRequest): Promise<WorkingPattern> {
  const { data: result } = await client.POST<WorkingPattern>("/v1/working-patterns", {
    body: data,
  });
  return result;
}

export async function updateWorkingPattern(
  patternId: string,
  data: UpdateWorkingPatternRequest
): Promise<WorkingPattern> {
  const { data: result } = await client.PUT<WorkingPattern>("/v1/working-patterns/{pattern_id}", {
    params: { path: { pattern_id: patternId } },
    body: data,
  });
  return result;
}

export async function fetchPatternDetails(patternId: string): Promise<WorkingPatternDetail[]> {
  const { data } = await client.GET<WorkingPatternDetail[]>("/v1/working-patterns/{pattern_id}", {
    params: {
      path: { pattern_id: patternId },
    },
  });
  return data;
}

export async function assignPattern(
  employmentId: string, 
  data: AssignPatternRequest
): Promise<EmployeeWorkingPattern> {
  const { data: result } = await client.POST<EmployeeWorkingPattern>("/v1/employments/{employment_id}/working-patterns", {
    params: {
      path: { employment_id: employmentId },
    },
    body: data,
  });
  return result;
}

export async function bulkAssignPattern(
  data: BulkAssignPatternRequest & { allow_warnings?: boolean; override_reason?: string }
): Promise<BulkAssignResponse> {
  const { data: result } = await client.POST<BulkAssignResponse>("/v1/working-patterns/bulk-assign", {
    body: data,
  });
  return result;
}

export async function fetchEmployeePatternHistory(
  employmentId: string
): Promise<EmployeeWorkingPattern[]> {
  const { data } = await client.GET<EmployeeWorkingPattern[]>("/v1/employments/{employment_id}/working-patterns", {
    params: {
      path: { employment_id: employmentId },
    },
  });
  return data;
}

export async function deleteWorkingPattern(patternId: string): Promise<void> {
  await client.DELETE("/v1/working-patterns/{pattern_id}", {
    params: {
      path: { pattern_id: patternId },
    },
  });
}
