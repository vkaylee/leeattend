import type { EmploymentPhaseDefinition, EmployeePhase } from "./types";

import { client } from "./client";

export async function fetchEmploymentPhases(wsId: string): Promise<EmploymentPhaseDefinition[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employment-phases", {
    params: { path: { ws_id: wsId } }
  });
  return data as unknown as EmploymentPhaseDefinition[];
}

export async function createEmploymentPhase(wsId: string, data: Partial<EmploymentPhaseDefinition>): Promise<EmploymentPhaseDefinition> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employment-phases", {
    params: { path: { ws_id: wsId } },
    body: data as unknown as never
  });
  if (!result) throw new Error("No response data");
  return result as unknown as EmploymentPhaseDefinition;
}

export async function fetchEmployeePhases(wsId: string, employeeId: string): Promise<EmployeePhase[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employees/{emp_id}/phases", { params: { path: { ws_id: wsId, emp_id: employeeId } } });
  if (!data) throw new Error("No response data");
  return data;
}

export async function assignEmployeePhase(
  wsId: string,
  employeeId: string,
  data: { phase_definition_id: string; start_date: string; end_date: string; override_config?: unknown }
): Promise<EmployeePhase> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employees/{emp_id}/phases", { params: { path: { ws_id: wsId, emp_id: employeeId } }, body: data as unknown as never });
  if (!result) throw new Error("No response data");
  return result;
}

export async function updateEmployeePhaseStatus(
  wsId: string,
  employeeId: string,
  phaseId: string,
  data: { status: string; evaluation_notes?: string }
): Promise<EmployeePhase> {
  const { data: result } = await client.PUT("/v1/workspaces/{ws_id}/employees/{emp_id}/phases/{phase_id}/status", {
    params: { path: { ws_id: wsId, emp_id: employeeId, phase_id: phaseId } },
    body: data
  });
  if (!result) throw new Error("No response data");
  return result;
}

export async function fetchEmploymentAssignedPhases(wsId: string, employmentId: string): Promise<EmployeePhase[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employments/{employment_id}/phases", {
    params: { path: { ws_id: wsId, employment_id: employmentId } }
  });
  if (!data) throw new Error("No response data");
  return data as unknown as EmployeePhase[];
}

export async function assignEmploymentPhase(
  wsId: string,
  employmentId: string,
  data: { phase_definition_id: string; start_date: string; end_date: string; override_config?: unknown }
): Promise<EmployeePhase> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employments/{employment_id}/phases", {
    params: { path: { ws_id: wsId, employment_id: employmentId } },
    body: data as unknown as never
  });
  if (!result) throw new Error("No response data");
  return result as unknown as EmployeePhase;
}

export async function updateEmploymentPhaseStatus(
  wsId: string,
  employmentId: string,
  phaseId: string,
  data: { status: string; evaluation_notes?: string }
): Promise<EmployeePhase> {
  const { data: result } = await client.PUT("/v1/workspaces/{ws_id}/employments/{employment_id}/phases/{phase_id}/status", {
    params: { path: { ws_id: wsId, employment_id: employmentId, phase_id: phaseId } },
    body: data
  });
  if (!result) throw new Error("No response data");
  return result as unknown as EmployeePhase;
}
