import type { components } from "@/types/api-generated";

import type {
  Employee,
  EmploymentPeriod,
  EmployeeDepartmentHistory,
  PaginatedResponse,
  Employment,
  CompensationHistoryResponse,
} from "./types";

// ── Typed request body aliases (from generated OpenAPI schema) ──────────────

export type CreateEmployeeRequest = components["schemas"]["CreateEmployeeRequest"];
export type UpdateEmployeeRequest = components["schemas"]["UpdateEmployeeRequest"];

// Employment is already exported from types.ts; re-exporting here causes
// a duplicate when both modules are barrel-re-exported together.

import { client, fetch as apiFetchPath } from "./client";

export * from "./employee-phases";
export * from "./employee-conditions";

export async function fetchEmployees(
  _wsId: string,
  page = 1,
  limit = 50,
  permission?: string
): Promise<PaginatedResponse<Employee>> {
  const { data } = await apiFetchPath<PaginatedResponse<Employee>>("GET", "/employees", {
    params: { query: { page, limit, permission } },
  });
  return data;
}

export interface SearchEmployeesParams {
  customFieldFilters?: string;
  departmentIds?: string[];
  limit?: number;
  page?: number;
  permission?: string;
  query: string;
  sortBy?: string;
  sortDir?: string;
  status?: string;
  wsId: string;
}

export async function searchEmployees(
  params: SearchEmployeesParams
): Promise<PaginatedResponse<Employee>> {
  const {
    query,
    page,
    limit,
    sortBy,
    sortDir,
    departmentIds,
    permission,
    customFieldFilters,
    status,
  } = params;

  const { data } = await apiFetchPath<PaginatedResponse<Employee>>("GET", "/employees", {
    params: {
      query: {
        page,
        limit,
        q: query || undefined,
        sort_by: sortBy ?? undefined,
        sort_dir: sortDir ?? undefined,
        department_id: (departmentIds && departmentIds.length > 0) ? departmentIds.join(",") : undefined,
        permission: permission ?? undefined,
        custom_field_filters: customFieldFilters ?? undefined,
        status: status ?? undefined,
      },
    },
  });
  return data;
}

export async function fetchEmployee(_wsId: string, id: string): Promise<Employee> {
  const { data } = await apiFetchPath<Employee>("GET", `/employees/${encodeURIComponent(id)}`);
  return data;
}

export async function fetchEmploymentHistory(_wsId: string, _id: string): Promise<EmploymentPeriod[]> { return Promise.resolve([]); }

export async function rehireEmployee(
  wsId: string,
  id: string,
  data: {
    hire_date: string;
    employer_id: string;
    department_id?: string | null;
    primary_location_id?: string | null;
  }
): Promise<Employee> {
  const { data: result } = await client.POST("/workspaces/{workspace_id}/employees/{id}/rehire", {
    params: {
      path: { workspace_id: wsId, id },
    },
    body: data,
  });
  return result;
}

export async function createEmployee(
  _wsId: string,
  employee: CreateEmployeeRequest,
): Promise<Employee> {
  const { data } = await apiFetchPath<Employee>("POST", "/employees", { body: employee });
  return data;
}

export async function updateEmployee(
  _wsId: string,
  id: string,
  employee: UpdateEmployeeRequest,
): Promise<Employee> {
  const { data } = await apiFetchPath<Employee>("PUT", `/employees/${encodeURIComponent(id)}`, { body: employee });
  return data;
}

export async function deleteEmployee(_wsId: string, id: string): Promise<void> {
  await apiFetchPath<undefined>("DELETE", `/employees/${encodeURIComponent(id)}`);
}

export async function transferEmployee(
  _wsId: string,
  employeeId: string,
  data: { department_id: string; effective_date: string; notes?: string }
): Promise<EmployeeDepartmentHistory> {
  const { data: result } = await apiFetchPath<EmployeeDepartmentHistory>("POST", `/employees/${encodeURIComponent(employeeId)}/departments`, { body: data });
  return result;
}

export async function fetchEmployeeDepartmentHistory(_wsId: string, employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const { data } = await apiFetchPath<EmployeeDepartmentHistory[]>("GET", `/employees/${encodeURIComponent(employeeId)}/departments`);
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
  return data;
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
  return data;
}

export async function fetchCompensationHistory(
  wsId: string,
  employeeId: string,
  employmentId?: string
): Promise<CompensationHistoryResponse> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/compensation-history", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
      // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- || intentional: empty-string from form inputs must be coerced to undefined
      query: { employment_id: employmentId || undefined },
    },
  });
  return data;
}

export type TerminationPreviewResponse = components["schemas"]["TerminationPreviewResponse"];
export type ConfirmTerminationRow = components["schemas"]["ConfirmTerminationRow"];

// Re-export termination-related functions from the split file
export {
  uploadTerminationImportPreview,
  confirmTerminationImport,
  downloadTerminationTemplate,
} from "./employees-termination";
