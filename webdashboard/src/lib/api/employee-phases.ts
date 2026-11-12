import { unwrapDataArray } from "@/types/api-overrides";

import type {
  EmploymentPhaseDefinition,
  EmployeePhase,
  CreateEmploymentPhaseDefinitionRequest,
  AssignEmployeePhaseRequest,
  UpdateEmployeePhaseStatusRequest,
} from "./types";

import { client } from "./client";

export interface EmploymentPhaseAssignment extends EmployeePhase {
  employee_id: string;
  employee_name: string;
  employee_code: string;
  department_name?: string | null;
  phase_name: string;
  phase_type: string;
}

export interface EmploymentPhaseAssignmentListResponse {
  data: EmploymentPhaseAssignment[];
  total: number;
  page: number;
  limit: number;
}

export interface EmploymentPhaseAssignmentListParams {
  employeeId?: string;
  limit?: number;
  page?: number;
  q?: string;
  status?: string;
}

export async function fetchEmploymentPhaseAssignments(
  params: EmploymentPhaseAssignmentListParams = {},
): Promise<EmploymentPhaseAssignmentListResponse> {
  const { employeeId, ...query } = params;
  const { data } = await client.GET<EmploymentPhaseAssignmentListResponse>("/api/v1/employment-phase-assignments", {
    params: {
      query: {
        ...query,
        employee_id: employeeId,
      },
    },
  });
  return data;
}

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
