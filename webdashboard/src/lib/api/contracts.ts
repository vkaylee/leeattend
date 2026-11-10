import type {
  CreateContractRequest,
  EmployeeContract,
  EmployeeContractWithEmployee,
  UpdateContractRequest,
} from "./types";
import { client } from "./client";


export async function fetchEmployeeContracts(employeeId: string): Promise<EmployeeContract[]> {
  const { data } = await client.GET<EmployeeContract[]>("/v1/employees/{employee_id}/contracts", {
    params: {
      path: { employee_id: employeeId },
    },
  });
  return data;
}

export async function fetchEmployeeContract(employeeId: string, contractId: string): Promise<EmployeeContract> {
  const { data } = await client.GET<EmployeeContract>("/v1/employees/{employee_id}/contracts/{contract_id}", {
    params: {
      path: { employee_id: employeeId, contract_id: contractId },
    },
  });
  return data;
}

export async function createEmployeeContract(
  employeeId: string,
  data: CreateContractRequest,
): Promise<EmployeeContract> {
  const { data: result } = await client.POST<EmployeeContract>("/v1/employees/{employee_id}/contracts", {
    params: {
      path: { employee_id: employeeId },
    },
    body: data,
  });
  return result;
}

export async function updateEmployeeContract(
  employeeId: string,
  contractId: string,
  data: UpdateContractRequest,
): Promise<EmployeeContract> {
  const { data: result } = await client.PATCH<EmployeeContract>("/v1/employees/{employee_id}/contracts/{contract_id}", {
    params: {
      path: { employee_id: employeeId, contract_id: contractId },
    },
    body: data,
  });
  return result;
}

export async function deleteEmployeeContract(employeeId: string, contractId: string): Promise<void> {
  await client.DELETE("/v1/employees/{employee_id}/contracts/{contract_id}", {
    params: {
      path: { employee_id: employeeId, contract_id: contractId },
    },
  });
}

export async function fetchContracts(): Promise<EmployeeContractWithEmployee[]> {
  const { data } = await client.GET<EmployeeContractWithEmployee[]>("/v1/contracts");
  return data;
}
