import type { EmployeeContract, CreateContractRequest, UpdateContractRequest } from "./contract-types";

import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";

export async function fetchEmployeeContracts(wsId: string, employeeId: string): Promise<EmployeeContract[]> {
  const response = await apiFetch(`${API_BASE_URL}/v1/workspaces/${wsId}/employees/${employeeId}/contracts`);
  if (!response.ok) throw new Error("Failed to fetch employee contracts");
  return response.json() as Promise<EmployeeContract[]>;
}

export async function fetchEmployeeContract(wsId: string, employeeId: string, contractId: string): Promise<EmployeeContract> {
  const response = await apiFetch(`${API_BASE_URL}/v1/workspaces/${wsId}/employees/${employeeId}/contracts/${contractId}`);
  if (!response.ok) throw new Error("Failed to fetch employee contract");
  return response.json() as Promise<EmployeeContract>;
}

export async function createEmployeeContract(
  wsId: string,
  employeeId: string,
  data: CreateContractRequest
): Promise<EmployeeContract> {
  const response = await apiFetch(`${API_BASE_URL}/v1/workspaces/${wsId}/employees/${employeeId}/contracts`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    let errMsg = "Failed to create employee contract";
    try {
      const errBody = await response.json() as { error?: { message?: string } | string };
      if (typeof errBody.error === 'string') {
        errMsg = errBody.error;
      } else if (errBody.error?.message) {
        errMsg = errBody.error.message;
      }
    } catch { /* ignore */ }
    throw new Error(errMsg);
  }
  return response.json() as Promise<EmployeeContract>;
}

export async function updateEmployeeContract(
  wsId: string,
  employeeId: string,
  contractId: string,
  data: UpdateContractRequest
): Promise<EmployeeContract> {
  const response = await apiFetch(`${API_BASE_URL}/v1/workspaces/${wsId}/employees/${employeeId}/contracts/${contractId}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    let errMsg = "Failed to update employee contract";
    try {
      const errBody = await response.json() as { error?: { message?: string } | string };
      if (typeof errBody.error === 'string') {
        errMsg = errBody.error;
      } else if (errBody.error?.message) {
        errMsg = errBody.error.message;
      }
    } catch { /* ignore */ }
    throw new Error(errMsg);
  }
  return response.json() as Promise<EmployeeContract>;
}

export async function deleteEmployeeContract(wsId: string, employeeId: string, contractId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/v1/workspaces/${wsId}/employees/${employeeId}/contracts/${contractId}`, {
    method: "DELETE",
  });
  if (!response.ok) {
    let errMsg = "Failed to delete employee contract";
    try {
      const errBody = await response.json() as { error?: { message?: string } | string };
      if (typeof errBody.error === 'string') {
        errMsg = errBody.error;
      } else if (errBody.error?.message) {
        errMsg = errBody.error.message;
      }
    } catch { /* ignore */ }
    throw new Error(errMsg);
  }
}
