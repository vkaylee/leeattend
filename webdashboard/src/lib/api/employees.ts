import type {
  Employee,
  EmploymentPeriod,
  EmployeeDepartmentHistory,
  WorkingCondition,
  PaginatedResponse,
  EmploymentPhaseDefinition,
  EmployeePhase
} from "./types";

// @ts-nocheck
import { client } from "./client";

export type {
  Employee,
  EmploymentPeriod,
  EmployeeDepartmentHistory,
  WorkingCondition,
  PaginatedResponse,
  EmploymentPhaseDefinition,
  EmployeePhase
};

export async function fetchEmployees(
  wsId: string,
  page = 1,
  limit = 50,
  permission?: string
): Promise<PaginatedResponse<Employee>> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees", {
    params: {
      path: { workspace_id: wsId },
      query: { page, limit, permission },
    },
  });
  if (!data) throw new Error("No response data");
  return data;
}

// --- Employment Phases API ---

export async function fetchEmploymentPhases(wsId: string): Promise<EmploymentPhaseDefinition[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employment-phases", {
            params: { path: { ws_id: wsId } }
      });
  return data as unknown as EmploymentPhaseDefinition[];
}

export async function createEmploymentPhase(wsId: string, data: Partial<EmploymentPhaseDefinition>): Promise<EmploymentPhaseDefinition> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employment-phases", {
            params: { path: { ws_id: wsId } },
          body: data as any });
  return result as unknown as EmploymentPhaseDefinition;
}

export async function fetchEmployeePhases(wsId: string, employeeId: string): Promise<EmployeePhase[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/employees/{emp_id}/phases", { params: { path: { ws_id: wsId, emp_id: employeeId } } });
  return data!;
}

export async function assignEmployeePhase(
  wsId: string,
  employeeId: string,
  data: { phase_definition_id: string; start_date: string; end_date: string; override_config?: unknown }
): Promise<EmployeePhase> {
  const { data: result } = await client.POST("/v1/workspaces/{ws_id}/employees/{emp_id}/phases", { params: { path: { ws_id: wsId, emp_id: employeeId } }, body: data as any });
  return result!;
}

export async function updateEmployeePhaseStatus(
  wsId: string,
  employeeId: string,
  phaseId: string,
  data: { status: string; evaluation_notes?: string }
): Promise<EmployeePhase> {
  const { data: result } = await client.PUT("/v1/workspaces/{ws_id}/employees/{emp_id}/phases/{phase_id}/status", {
            params: { path: { ws_id: wsId, emp_id: employeeId, phase_id: phaseId } },
          body: data });
  return result!;
}

export async function fetchWorkingConditions(wsId: string, employeeId: string): Promise<WorkingCondition[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/conditions", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as WorkingCondition[];
}

export async function createWorkingCondition(
  wsId: string,
  employeeId: string,
  condition: Partial<WorkingCondition>
): Promise<WorkingCondition> {
  const { data } = await client.POST("/workspaces/{workspace_id}/employees/{id}/conditions", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
    body: condition as any,
  });
  if (!data) throw new Error("No response data");
  return data as unknown as WorkingCondition;
}

export async function deleteWorkingCondition(wsId: string, employeeId: string, conditionId: string): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/employees/{id}/conditions/{cond_id}", {
    params: {
      path: { workspace_id: wsId, id: employeeId, cond_id: conditionId },
    },
  });
}

export interface SearchEmployeesParams {
  departmentIds?: string[];
  limit?: number;
  page?: number;
  query: string;
  sortBy?: string;
  sortDir?: string;
  wsId: string;
}

export async function searchEmployees(
  params: SearchEmployeesParams
): Promise<PaginatedResponse<Employee>> {
  const {
    wsId,
    query,
    page = 1,
    limit = 50,
    sortBy,
    sortDir,
    departmentIds,
  } = params;

  const { data } = await client.GET("/workspaces/{workspace_id}/employees", {
    params: {
      path: { workspace_id: wsId },
      query: {
        page,
        limit,
        q: query || undefined,
        sort_by: sortBy || undefined,
        sort_dir: sortDir || undefined,
        department_id: (departmentIds && departmentIds.length > 0) ? departmentIds.join(",") : undefined,
      },
    },
  });
  if (!data) throw new Error("No response data");
  return data;
}

export async function fetchEmployee(wsId: string, id: string): Promise<Employee> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
  });
  if (!data) throw new Error("No response data");
  return data;
}

export async function fetchEmploymentHistory(
  _wsId: string,
  _id: string
): Promise<EmploymentPeriod[]> {
  return Promise.resolve([]);
}

export async function rehireEmployee(
  wsId: string,
  id: string,
  data: { hire_date: string }
): Promise<Employee> {
  const { data: result } = await client.POST("/workspaces/{workspace_id}/employees/{id}/rehire", {
    params: {
      path: { workspace_id: wsId, id },
    },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result;
}

export async function createEmployee(
  wsId: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const { data } = await client.POST("/workspaces/{workspace_id}/employees", {
    params: {
      path: { workspace_id: wsId },
    },
    body: employee as any,
  });
  if (!data) throw new Error("No response data");
  return data;
}

export async function updateEmployee(
  wsId: string,
  id: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const { data } = await client.PUT("/workspaces/{workspace_id}/employees/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
    body: employee as any,
  });
  if (!data) throw new Error("No response data");
  return data;
}

export async function deleteEmployee(wsId: string, id: string): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/employees/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
  });
}

export async function transferEmployee(
  wsId: string,
  employeeId: string,
  data: { department_id: string; effective_date: string; notes?: string }
): Promise<EmployeeDepartmentHistory> {
  const { data: result } = await client.POST("/workspaces/{workspace_id}/employees/{id}/departments", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
    body: data,
  });
  if (!result) throw new Error("No response data");
  return result;
}

export async function fetchEmployeeDepartmentHistory(wsId: string, employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/departments", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
  });
  if (!data) throw new Error("No response data");
  return data;
}