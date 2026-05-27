import type { EmployeeContract, CreateContractRequest, UpdateContractRequest } from "./contract-types";

import { apiClient } from "./client";

export async function fetchEmployeeContracts(wsId: string, employeeId: string): Promise<EmployeeContract[]> {
  const { data } = await apiClient.get<EmployeeContract[]>(`/v1/workspaces/${wsId}/employees/${employeeId}/contracts`);
  return data;
}

export async function fetchEmployeeContract(wsId: string, employeeId: string, contractId: string): Promise<EmployeeContract> {
  const { data } = await apiClient.get<EmployeeContract>(`/v1/workspaces/${wsId}/employees/${employeeId}/contracts/${contractId}`);
  return data;
}

export async function createEmployeeContract(
  wsId: string,
  employeeId: string,
  data: CreateContractRequest
): Promise<EmployeeContract> {
  const { data: result } = await apiClient.post<EmployeeContract>(`/v1/workspaces/${wsId}/employees/${employeeId}/contracts`, data);
  return result;
}

export async function updateEmployeeContract(
  wsId: string,
  employeeId: string,
  contractId: string,
  data: UpdateContractRequest
): Promise<EmployeeContract> {
  const { data: result } = await apiClient.patch<EmployeeContract>(`/v1/workspaces/${wsId}/employees/${employeeId}/contracts/${contractId}`, data);
  return result;
}

export async function deleteEmployeeContract(wsId: string, employeeId: string, contractId: string): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/employees/${employeeId}/contracts/${contractId}`);
}
