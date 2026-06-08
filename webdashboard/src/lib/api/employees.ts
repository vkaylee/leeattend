import type {
  Employee,
  EmploymentPeriod,
  EmployeeDepartmentHistory,
  PaginatedResponse,
  Employment,
} from "./types";

import { components } from "@/types/api-generated";
import { client } from "./client";

export * from "./employee-phases";
export * from "./employee-conditions";

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

export interface SearchEmployeesParams {
  departmentIds?: string[];
  limit?: number;
  page?: number;
  permission?: string;
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
    permission,
  } = params;

  const { data } = await client.GET("/workspaces/{workspace_id}/employees", {
    params: {
      path: { workspace_id: wsId },
      query: {
        page,
        limit,
        q: query || undefined,
        sort_by: sortBy ?? undefined,
        sort_dir: sortDir ?? undefined,
        department_id: (departmentIds && departmentIds.length > 0) ? departmentIds.join(",") : undefined,
        permission: permission ?? undefined,
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
    body: employee as unknown as never,
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
    body: employee as unknown as never,
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


export async function fetchEmployeeEmployments(
  wsId: string,
  employeeId: string
): Promise<Employment[]> {
  const { data } = await client.GET("/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments", {
    params: {
      path: { ws_id: wsId, employee_id: employeeId },
    },
  });
  if (!data) throw new Error("No response data");
  return data as unknown as Employment[];
}

export async function updateEmployeeEmployment(
  wsId: string,
  employeeId: string,
  employmentId: string,
  body: components["schemas"]["UpdateEmploymentRequest"]
): Promise<Employment> {
  const { data } = await client.PATCH("/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}", {
    params: {
      path: { ws_id: wsId, employee_id: employeeId, employment_id: employmentId },
    },
    body,
  });
  if (!data) throw new Error("No response data");
  return data as unknown as Employment;
}