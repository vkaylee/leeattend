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
 
export type { Employment } from "./types";

import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

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
    wsId,
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
        custom_field_filters: customFieldFilters ?? undefined,
        status: status ?? undefined,
      },
    },
  });
  return data;
}

export async function fetchEmployee(wsId: string, id: string): Promise<Employee> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
  });
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
  wsId: string,
  employee: CreateEmployeeRequest,
): Promise<Employee> {
  const { data } = await client.POST("/workspaces/{workspace_id}/employees", {
    params: {
      path: { workspace_id: wsId },
    },
    body: employee,
  });
  return data;
}

export async function updateEmployee(
  wsId: string,
  id: string,
  employee: UpdateEmployeeRequest,
): Promise<Employee> {
  const { data } = await client.PUT("/workspaces/{workspace_id}/employees/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
    body: employee,
  });
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
  return result;
}

export async function fetchEmployeeDepartmentHistory(wsId: string, employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/departments", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
  });
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
      // || intentional: empty-string employment_id from form inputs must be coerced to undefined
      query: { employment_id: employmentId || undefined },
    },
  });
  return data;
}

export type TerminationPreviewResponse = components["schemas"]["TerminationPreviewResponse"];
export type ConfirmTerminationRow = components["schemas"]["ConfirmTerminationRow"];

/** Upload Excel file to preview employee bulk terminations.
 *  Uses apiFetch directly because multipart/FormData is not modeled in the
 *  generated OpenAPI types. */
export async function uploadTerminationImportPreview(
  wsId: string,
  file: File
): Promise<TerminationPreviewResponse> {
  const formData = new FormData();
  formData.append("file", file);
  const res = await apiFetch(`/v1/workspaces/${wsId}/employees/import-termination/preview`, {
    method: "POST",
    body: formData,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
  return res.json() as Promise<TerminationPreviewResponse>;
}

/** Confirm employee bulk terminations */
export async function confirmTerminationImport(
  wsId: string,
  rows: ConfirmTerminationRow[]
): Promise<{ success_count: number; job_id?: string }> {
  const { data } = await client.POST("/workspaces/{workspace_id}/employees/import-termination/confirm", {
    params: {
      path: { workspace_id: wsId },
    },
    body: {
      rows,
    },
  });
  return data as { success_count: number; job_id?: string };
}

/** Download bulk termination Excel template via authenticated fetch */
export async function downloadTerminationTemplate(wsId: string, lang = "en"): Promise<void> {
  const response = await apiFetch(`/v1/workspaces/${wsId}/employees/import-termination/template?lang=${lang}`);
  if (!response.ok) throw new Error("Failed to download template");
  const blob = await response.blob();
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = lang === "vi"
    ? "mau_import_nghi_viec.xlsx"
    : lang === "zh"
    ? "employee_termination_template_zh.xlsx"
    : "employee_termination_template.xlsx";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  window.URL.revokeObjectURL(url);
}