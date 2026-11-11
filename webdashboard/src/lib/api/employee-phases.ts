import { unwrapDataArray } from "@/types/api-overrides";

import type {
  EmploymentPhaseDefinition,
  EmployeePhase,
  CreateEmploymentPhaseDefinitionRequest,
  AssignEmployeePhaseRequest,
  UpdateEmployeePhaseStatusRequest,
} from "./types";

import { client } from "./client";

export async function fetchEmploymentPhases(): Promise<EmploymentPhaseDefinition[]> {
  const { data } = await client.GET<EmploymentPhaseDefinition[]>("/api/v1/employment-phases");
  return unwrapDataArray<EmploymentPhaseDefinition>(data);
}

export async function createEmploymentPhase(
  data: CreateEmploymentPhaseDefinitionRequest,
): Promise<EmploymentPhaseDefinition> {
  const { data: result } = await client.POST<EmploymentPhaseDefinition>("/api/v1/employment-phases", {
    body: data,
  });
  return result;
}

export async function fetchEmployeePhases(employeeId: string): Promise<EmployeePhase[]> {
  const { data } = await client.GET<EmployeePhase[]>("/api/v1/employees/{user_id}/phases", {
    params: { path: { user_id: employeeId } },
  });
  return unwrapDataArray<EmployeePhase>(data);
}

export async function assignEmployeePhase(
  employeeId: string,
  data: AssignEmployeePhaseRequest,
): Promise<EmployeePhase> {
  const { data: result } = await client.POST<EmployeePhase>("/api/v1/employees/{user_id}/phases", {
    params: { path: { user_id: employeeId } },
    body: data,
  });
  return result;
}

export async function updateEmployeePhaseStatus(
  employeeId: string,
  phaseId: string,
  data: UpdateEmployeePhaseStatusRequest,
): Promise<EmployeePhase> {
  const { data: result } = await client.PUT<EmployeePhase>("/api/v1/employees/{user_id}/phases/{phase_id}/status", {
    params: { path: { user_id: employeeId, phase_id: phaseId } },
    body: data,
  });
  return result;
}

export async function fetchEmploymentAssignedPhases(employmentId: string): Promise<EmployeePhase[]> {
  const { data } = await client.GET<EmployeePhase[]>("/api/v1/employments/{employment_id}/phases", {
    params: { path: { employment_id: employmentId } },
  });
  return unwrapDataArray<EmployeePhase>(data);
}

export async function assignEmploymentPhase(
  employmentId: string,
  data: AssignEmployeePhaseRequest,
): Promise<EmployeePhase> {
  const { data: result } = await client.POST<EmployeePhase>("/api/v1/employments/{employment_id}/phases", {
    params: { path: { employment_id: employmentId } },
    body: data,
  });
  return result;
}

export async function updateEmploymentPhaseStatus(
  employmentId: string,
  phaseId: string,
  data: UpdateEmployeePhaseStatusRequest,
): Promise<EmployeePhase> {
  const { data: result } = await client.PUT<EmployeePhase>("/api/v1/employments/{employment_id}/phases/{phase_id}/status", {
    params: { path: { employment_id: employmentId, phase_id: phaseId } },
    body: data,
  });
  return result;
}
