import type {
  EmploymentPhaseDefinition,
  EmployeePhase,
  CreateEmploymentPhaseDefinitionRequest,
  AssignEmployeePhaseRequest,
  UpdateEmployeePhaseStatusRequest,
} from "./types";

import { client } from "./client";

export async function fetchEmploymentPhases(wsId: string): Promise<EmploymentPhaseDefinition[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employment-phases", {
    params: { path: { ws_id: wsId } }
  });
  return (data as EmploymentPhaseDefinition[]) ?? [];
}

export async function createEmploymentPhase(
  wsId: string,
  data: CreateEmploymentPhaseDefinitionRequest
): Promise<EmploymentPhaseDefinition> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employment-phases", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  return result as EmploymentPhaseDefinition;
}

export async function fetchEmployeePhases(wsId: string, employeeId: string): Promise<EmployeePhase[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employees/{emp_id}/phases", {
    params: { path: { ws_id: wsId, emp_id: employeeId } }
  });
  if (!data) return [];
  return data;
}

export async function assignEmployeePhase(
  wsId: string,
  employeeId: string,
  data: AssignEmployeePhaseRequest
): Promise<EmployeePhase> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employees/{emp_id}/phases", {
    params: { path: { ws_id: wsId, emp_id: employeeId } },
    body: data
  });
  return result;
}

export async function updateEmployeePhaseStatus(
  wsId: string,
  employeeId: string,
  phaseId: string,
  data: UpdateEmployeePhaseStatusRequest
): Promise<EmployeePhase> {
  const { data: result } = await client.PUT("/v1/workspaces/{ws_id}/employees/{emp_id}/phases/{phase_id}/status", {
    params: { path: { ws_id: wsId, emp_id: employeeId, phase_id: phaseId } },
    body: data
  });
  return result;
}

export async function fetchEmploymentAssignedPhases(wsId: string, employmentId: string): Promise<EmployeePhase[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employments/{employment_id}/phases", {
    params: { path: { ws_id: wsId, employment_id: employmentId } }
  });
  if (!data) return [];
  return data;
}

export async function assignEmploymentPhase(
  wsId: string,
  employmentId: string,
  data: AssignEmployeePhaseRequest
): Promise<EmployeePhase> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employments/{employment_id}/phases", {
    params: { path: { ws_id: wsId, employment_id: employmentId } },
    body: data
  });
  return result;
}

export async function updateEmploymentPhaseStatus(
  wsId: string,
  employmentId: string,
  phaseId: string,
  data: UpdateEmployeePhaseStatusRequest
): Promise<EmployeePhase> {
  const { data: result } = await client.PUT("/v1/workspaces/{ws_id}/employments/{employment_id}/phases/{phase_id}/status", {
    params: { path: { ws_id: wsId, employment_id: employmentId, phase_id: phaseId } },
    body: data
  });
  return result;
}
