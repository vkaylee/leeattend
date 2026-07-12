import type { EmployeeContract, CreateContractRequest, UpdateContractRequest } from "./contract-types";

import { client } from "./client";


export async function fetchEmployeeContracts(wsId: string, employeeId: string): Promise<EmployeeContract[]> {
  const { data } = await client.GET("/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts", {
    params: {
      path: { ws_id: wsId, employee_id: employeeId },
    },
  });
  return data;
}

export async function fetchEmployeeContract(wsId: string, employeeId: string, contractId: string): Promise<EmployeeContract> {
  const { data } = await client.GET("/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", {
    params: {
      path: { ws_id: wsId, employee_id: employeeId, contract_id: contractId },
    },
  });
  return data;
}

export async function createEmployeeContract(
  wsId: string,
  employeeId: string,
  data: CreateContractRequest
): Promise<EmployeeContract> {
  const { data: result } = await client.POST("/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts", {
    params: {
      path: { ws_id: wsId, employee_id: employeeId },
    },
    body: data,
  });
  return result;
}

export async function updateEmployeeContract(
  wsId: string,
  employeeId: string,
  contractId: string,
  data: UpdateContractRequest
): Promise<EmployeeContract> {
  const { data: result } = await client.PATCH("/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", {
    params: {
      path: { ws_id: wsId, employee_id: employeeId, contract_id: contractId },
    },
    body: data,
  });
  return result;
}

export async function deleteEmployeeContract(wsId: string, employeeId: string, contractId: string): Promise<void> {
  await client.DELETE("/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", {
    params: {
      path: { ws_id: wsId, employee_id: employeeId, contract_id: contractId },
    },
  });
}

export type EmployeeContractWithEmployee = EmployeeContract & {
  employee_code: string;
  employee_id: string;
  employee_name: string;
};

export async function fetchWorkspaceContracts(wsId: string): Promise<EmployeeContractWithEmployee[]> {
  const { data } = await client.GET("/api/v1/workspaces/{ws_id}/contracts", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return data;
}

