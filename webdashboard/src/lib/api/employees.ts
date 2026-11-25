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
export type MatchEmployeeRequest = components["schemas"]["MatchEmployeeRequest"];
export type MatchEmployeeResponse = components["schemas"]["MatchEmployeeResponse"];
export type EmployeeMatchCandidate = components["schemas"]["EmployeeMatchCandidate"];
export type CreateEmployeeTenureRequest = components["schemas"]["CreateEmployeeTenureRequest"];
export type EmployeeTenureCreated = components["schemas"]["EmployeeTenureCreated"];
export type EmployeeTenureRecord = components["schemas"]["EmployeeTenureRecord"];
export type CloseEmployeeTenureRequest = components["schemas"]["CloseEmployeeTenureRequest"];
export type EmployeeTenureClosed = components["schemas"]["EmployeeTenureClosed"];
export type TerminateEmploymentRequest = components["schemas"]["TerminateEmploymentRequest"];
export type TerminateEmploymentResult = components["schemas"]["TerminateEmploymentResult"];

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
  const { data } = await client.GET<PaginatedResponse<Employee>>("/api/v1/employees", {
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

  const { data } = await client.GET<PaginatedResponse<Employee>>("/api/v1/employees", {
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
  const { data } = await client.GET<Employee>("/api/v1/employees/{id}", {
    params: { path: { id } },
  });
  return data;
}

export async function fetchEmploymentHistory(_id: string): Promise<EmploymentPeriod[]> {
  return Promise.resolve([]);
}

export async function fetchEmployeeTenures(
  employeeId: string,
): Promise<EmployeeTenureRecord[]> {
  const { data } = await client.GET<EmployeeTenureRecord[]>("/api/v1/employees/{employee_id}/tenures", {
    params: { path: { employee_id: employeeId } },
  });
  return data;
}

export async function closeEmployeeTenure(
  employeeId: string,
  tenureId: string,
  body: CloseEmployeeTenureRequest,
): Promise<EmployeeTenureClosed> {
  const { data } = await client.POST<EmployeeTenureClosed>("/api/v1/employees/{employee_id}/tenures/{tenure_id}/close", {
    params: { path: { employee_id: employeeId, tenure_id: tenureId } },
    body,
  });
  return data;
}

export async function terminateEmployeeEmployment(
  employeeId: string,
  employmentId: string,
  body: TerminateEmploymentRequest,
): Promise<TerminateEmploymentResult> {
  const { data } = await client.POST<TerminateEmploymentResult>(
    "/api/v1/employees/{employee_id}/employments/{employment_id}/terminate",
    {
      params: { path: { employee_id: employeeId, employment_id: employmentId } },
      body,
    },
  );
  return data;
}

export async function createEmployee(
  employee: CreateEmployeeRequest,
): Promise<Employee> {
  const { data } = await client.POST<Employee>("/api/v1/employees", { body: employee });
  return data;
}

export async function matchEmployeeIdentity(
  body: MatchEmployeeRequest,
): Promise<MatchEmployeeResponse> {
  const { data } = await client.POST<MatchEmployeeResponse>("/api/v1/employees/match", { body });
  return data;
}

export async function createEmployeeTenure(
  employeeId: string,
  body: CreateEmployeeTenureRequest,
): Promise<EmployeeTenureCreated> {
  const { data } = await client.POST<EmployeeTenureCreated>("/api/v1/employees/{employee_id}/tenures", {
    params: { path: { employee_id: employeeId } },
    body,
  });
  return data;
}

export async function updateEmployee(
  id: string,
  employee: UpdateEmployeeRequest,
): Promise<Employee> {
  const { data } = await client.PUT<Employee>("/api/v1/employees/{id}", {
    params: { path: { id } },
    body: employee,
  });
  return data;
}

export async function deleteEmployee(id: string): Promise<void> {
  await client.DELETE("/api/v1/employees/{id}", {
    params: { path: { id } },
  });
}

export async function transferEmployee(
  employeeId: string,
  data: { department_id: string; effective_date: string; notes?: string },
): Promise<EmployeeDepartmentHistory> {
  const { data: result } = await client.POST<EmployeeDepartmentHistory>("/api/v1/employees/{id}/departments", {
    params: { path: { id: employeeId } },
    body: data,
  });
  return result;
}

export async function fetchEmployeeDepartmentHistory(employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const { data } = await client.GET<EmployeeDepartmentHistory[]>("/api/v1/employees/{id}/departments", {
    params: { path: { id: employeeId } },
  });
  return data;
}

export async function fetchEmployeeEmployments(
  employeeId: string,
): Promise<Employment[]> {
  const { data } = await client.GET<Employment[]>("/api/v1/employees/{employee_id}/employments", {
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
  const { data } = await client.PATCH<Employment>("/api/v1/employees/{employee_id}/employments/{employment_id}", {
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
  const { data } = await client.GET<CompensationHistoryResponse>("/api/v1/employees/{id}/compensation-history", {
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
