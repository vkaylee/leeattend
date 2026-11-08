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

import { client } from "./client";

export * from "./employee-phases";
export * from "./employee-conditions";

export async function fetchEmployees(
  page = 1,
  limit = 50,
  permission?: string,
): Promise<PaginatedResponse<Employee>> {
  const { data } = await client.GET<PaginatedResponse<Employee>>("/v1/employees", {
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
  wsId?: string;
}

export async function searchEmployees(
  params: SearchEmployeesParams,
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

  const { data } = await client.GET<PaginatedResponse<Employee>>("/v1/employees", {
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

export async function fetchEmployee(id: string): Promise<Employee> {
  const { data } = await client.GET<Employee>("/v1/employees/{id}", {
    params: { path: { id } },
  });
  return data;
}

export async function fetchEmploymentHistory(_id: string): Promise<EmploymentPeriod[]> {
  return Promise.resolve([]);
}

export interface RehireEmployeeRequest {
  hire_date: string;
  employer_id: string;
  department_id?: string | null;
  primary_location_id?: string | null;
}

export async function rehireEmployee(
  id: string,
  data: RehireEmployeeRequest,
): Promise<Employee> {
  const { data: result } = await client.POST<Employee>("/v1/employees/{id}/rehire", {
    params: {
      path: { id },
    },
    body: data,
  });
  return result;
}

export async function createEmployee(
  employee: CreateEmployeeRequest,
): Promise<Employee> {
  const { data } = await client.POST<Employee>("/v1/employees", { body: employee });
  return data;
}

export async function updateEmployee(
  id: string,
  employee: UpdateEmployeeRequest,
): Promise<Employee> {
  const { data } = await client.PUT<Employee>("/v1/employees/{id}", {
    params: { path: { id } },
    body: employee,
  });
  return data;
}

export async function deleteEmployee(id: string): Promise<void> {
  await client.DELETE("/v1/employees/{id}", {
    params: { path: { id } },
  });
}

export async function transferEmployee(
  employeeId: string,
  data: { department_id: string; effective_date: string; notes?: string },
): Promise<EmployeeDepartmentHistory> {
  const { data: result } = await client.POST<EmployeeDepartmentHistory>("/v1/employees/{id}/departments", {
    params: { path: { id: employeeId } },
    body: data,
  });
  return result;
}

export async function fetchEmployeeDepartmentHistory(employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const { data } = await client.GET<EmployeeDepartmentHistory[]>("/v1/employees/{id}/departments", {
    params: { path: { id: employeeId } },
  });
  return data;
}

export async function fetchEmployeeEmployments(
  employeeId: string,
): Promise<Employment[]> {
  const { data } = await client.GET<Employment[]>("/v1/employees/{employee_id}/employments", {
    params: {
      path: { employee_id: employeeId },
    },
  });
  return data;
}

export type UpdateEmploymentRequest = components["schemas"]["UpdateEmploymentRequest"];

export async function updateEmployeeEmployment(
  employeeId: string,
  employmentId: string,
  body: UpdateEmploymentRequest,
): Promise<Employment> {
  const { data } = await client.PATCH<Employment>("/v1/employees/{employee_id}/employments/{employment_id}", {
    params: {
      path: { employee_id: employeeId, employment_id: employmentId },
    },
    body,
  });
  return data;
}

export async function fetchCompensationHistory(
  employeeId: string,
  employmentId?: string,
): Promise<CompensationHistoryResponse> {
  const { data } = await client.GET<CompensationHistoryResponse>("/v1/employees/{id}/compensation-history", {
    params: {
      path: { id: employeeId },
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
