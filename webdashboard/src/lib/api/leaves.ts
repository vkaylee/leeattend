/* eslint-disable max-lines -- API wrapper: 400+ lines of thin exported functions; splitting would scatter imports */
import type { components } from "@/types/api-generated";

import type { LeavePolicyAssignmentDetail, LeaveRequest, EmployeeLeaveBalance, WorkspaceLeaveBalance, LeaveHistoryItem, LeavePolicy, LeaveType, LeaveTypeConfig, PaginatedResponse } from './types';

import { createBehalfAssignmentApi } from "./behalf-assignment-factory";
import { client, fetch } from "./client";
import { API_BASE_URL } from './config';
import { downloadBlob } from "./download-helper";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

// ── Typed request body aliases (from generated OpenAPI schema) ──────────────
// These replace handwritten inline types that required `as never` casts.
// Source: components["schemas"] in api-generated.ts

type SubmitLeaveRequestBody = components["schemas"]["SubmitLeaveRequest"];
type ApproveLeaveRequestBody = components["schemas"]["ApproveLeaveRequest"];
type AdjustBalanceRequestBody = components["schemas"]["AdjustBalanceRequest"];
type CreateLeaveTypeRequestBody = components["schemas"]["CreateLeaveTypeRequest"];
type AssignLeavePolicyRequestBody = components["schemas"]["AssignLeavePolicyRequest"];
type TriggerAccrualRequestBody = components["schemas"]["TriggerAccrualRequest"];
export type CreateLeavePolicyRequestBody = components["schemas"]["CreateLeavePolicyRequest"];
export type UpdateLeavePolicyRequestBody = components["schemas"]["UpdateLeavePolicyRequest"];

export async function submitLeaveRequest(
  wsId: string,
  data: SubmitLeaveRequestBody,
): Promise<LeaveRequest> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/leave/submit", {
    params: {
      path: { ws_id: wsId },
    },
    body: data,
  });
  return resData;
}

export async function fetchWorkspaceLeaveRequests(
  wsId: string,
  params?: {
    status?: "pending" | "approved" | "rejected" | "cancelled" | "pre_approved";
    employee_id?: string;
    page?: number;
    limit?: number;
  }
): Promise<PaginatedResponse<LeaveRequest>> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/requests", {
    params: {
      path: { ws_id: wsId },
      query: {
        status: params?.status,
        employee_id: params?.employee_id === "all" ? undefined : params?.employee_id,
        page: params?.page,
        limit: params?.limit,
      },
    },
  });
  return resData;
}

export async function fetchMyLeaveRequests(
  employeeId: string
): Promise<LeaveRequest[]> {
  const { data: resData } = await client.GET("/employees/{id}/leave/requests", {
    params: {
      path: { id: employeeId },
    },
  });
  return resData;
}

export async function approveLeaveRequest(
  wsId: string,
  requestId: string,
  data: ApproveLeaveRequestBody,
): Promise<void> {
  await client.POST("/workspaces/{ws_id}/leave/requests/{id}/approve", {
    params: {
      path: { ws_id: wsId, id: requestId },
    },
    body: data,
  });
}

export async function fetchEmployeeLeaveBalances(
  wsId: string,
  employeeId: string,
): Promise<EmployeeLeaveBalance[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/employees/{id}/leave/balances", {
    params: {
      path: { ws_id: wsId, id: employeeId },
    },
  });
  return resData;
}

export async function fetchWorkspaceLeaveBalances(
  wsId: string,
  params?: { page?: number; limit?: number; q?: string; year?: number }
): Promise<PaginatedResponse<WorkspaceLeaveBalance>> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/balances", {
    params: {
      path: { ws_id: wsId },
      query: {
              page: params?.page,
              limit: params?.limit,
              q: params?.q,
              year: params?.year,
            },
    },
  });
  return resData;
}

export async function fetchLeaveHistory(
  wsId: string,
  employeeId: string,
  leaveTypeId?: string,
): Promise<LeaveHistoryItem[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/employees/{id}/leave/history", {
    params: {
      path: { ws_id: wsId, id: employeeId },
      query: leaveTypeId ? { leave_type_id: leaveTypeId } : undefined,
    },
  });
  return resData;
}

export async function adjustLeaveBalance(
  wsId: string,
  employeeId: string,
  data: AdjustBalanceRequestBody,
): Promise<void> {
  await client.POST("/workspaces/{ws_id}/employees/{id}/leave/adjust", {
    params: {
      path: { ws_id: wsId, id: employeeId },
    },
    body: data,
  });
}

export async function fetchLeavePolicies(wsId: string): Promise<LeavePolicy[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/policies", {
    params: { path: { ws_id: wsId } },
  });
  return resData as LeavePolicy[];
}

export async function createLeavePolicy(
  wsId: string,
  data: CreateLeavePolicyRequestBody
): Promise<LeavePolicy> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/leave/policies", {
    params: { path: { ws_id: wsId } },
    body: data,
  });
  return resData as LeavePolicy;
}

export async function updateLeavePolicy(
  wsId: string,
  policyId: string,
  data: UpdateLeavePolicyRequestBody
): Promise<LeavePolicy> {
  const { data: resData } = await client.PATCH("/workspaces/{ws_id}/leave/policies/{policy_id}", {
    params: { path: { ws_id: wsId, policy_id: policyId } },
    body: data,
  });
  return resData as LeavePolicy;
}

export async function deleteLeavePolicy(
  wsId: string,
  policyId: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/leave/policies/{policy_id}", {
    params: {
      path: { ws_id: wsId, policy_id: policyId },
    },
  });
}

export async function fetchLeaveTypes(wsId: string): Promise<LeaveType[]> {
  const { data: resData } = await client.GET("/workspaces/{workspace_id}/leave/types", {
    params: {
      path: { workspace_id: wsId },
    },
  });
  return resData;
}

export async function createLeaveType(
  wsId: string,
  data: CreateLeaveTypeRequestBody
): Promise<LeaveType> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/leave/types", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data,
  });
  return resData;
}

export async function deleteLeaveType(
  wsId: string,
  id: string
): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/leave/types/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
  });
}

export async function restoreLeaveType(
  wsId: string,
  id: string
): Promise<LeaveType> {
  const { data: resData } = await client.POST(
    "/workspaces/{workspace_id}/leave/types/{id}/restore",
    {
      params: {
        path: { workspace_id: wsId, id },
      },
    }
  );
  return resData;
}

export async function fetchLeavePolicyAssignments(wsId: string): Promise<LeavePolicyAssignmentDetail[]> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/policies/assignments", {
    params: { path: { ws_id: wsId } },
  });
  return resData as LeavePolicyAssignmentDetail[];
}

export async function assignLeavePolicy(
  wsId: string,
  data: AssignLeavePolicyRequestBody
): Promise<void> {
  await client.POST("/workspaces/{ws_id}/leave/policies/assign", {
    params: {
      path: { ws_id: wsId },
    },
    body: data,
  });
}

export async function deleteLeavePolicyAssignment(
  wsId: string,
  assignmentId: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/leave/policies/assignments/{assignment_id}", {
    params: {
      path: { ws_id: wsId, assignment_id: assignmentId },
    },
  });
}

export interface AccrualJobResponse {
  job_id: string;
  message: string;
  status: string;
}

export async function triggerAccrualRun(
  wsId: string,
  data: TriggerAccrualRequestBody
): Promise<AccrualJobResponse> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/leave/accrual/run", {
    params: {
      path: { ws_id: wsId },
    },
    body: data,
  });
  return resData;
}

export interface JobStatusResponse {
  completed_at: string | null;
  error_message: string | null;
  job_id: string;
  status: string;
}

export async function fetchJobStatus(
  wsId: string,
  jobId: string
): Promise<JobStatusResponse> {
  const { data: resData } = await fetch<JobStatusResponse>("GET", "/workspaces/{ws_id}/jobs/{job_id}/status", {
    params: { path: { ws_id: wsId, job_id: jobId } },
  });
  return resData;
}

export async function updateLeaveTypeConfig(
  wsId: string,
  id: string,
  config: LeaveTypeConfig
): Promise<LeaveType> {
  const { data: resData } = await client.PUT("/workspaces/{workspace_id}/leave/types/{id}/config", {
    params: {
      path: { workspace_id: wsId, id },
    },
    body: { policy_config: config },
  });
  return resData;
}

// ========================
// Leave Import API
// ========================

export interface LeaveImportJob {
  created_at: string;
  error_log: string | null;
  id: string;
  processed_rows: number;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  total_rows: number;
  updated_at: string;
}

/** Upload an Excel file for bulk leave request import. */
export async function uploadLeaveExcel(
  wsId: string,
  file: File,
): Promise<{ job_id: string; status: string }> {
  const formData = new FormData();
  formData.append('file', file);
  const { data: resData } = await fetch<{ job_id: string; status: string }>("POST", "/workspaces/{ws_id}/leave/upload-excel", {
    params: { path: { ws_id: wsId } },
    body: formData,
  });
  return resData;
}

/** Fetch the latest leave import job for the workspace (page resume). */
export async function fetchLatestLeaveImport(
  wsId: string,
): Promise<LeaveImportJob | null> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/import/latest", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return resData as LeaveImportJob | null;
}

/** Fetch the latest leave import job (for polling). */
export async function fetchLeaveImportStatus(
  wsId: string,
): Promise<LeaveImportJob | null> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/leave/import/latest", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return resData as LeaveImportJob | null;
}

/** Get the URL for downloading the error Excel file (URL-only, for non-fetch usage). */
export function getLeaveImportErrorsUrl(wsId: string, jobId: string): string {
  return `${API_BASE_URL}/v1/workspaces/${wsId}/leave/import/${jobId}/errors`;
}

/** Get the URL for downloading the import template Excel (URL-only, for non-fetch usage). */
export function getLeaveImportTemplateUrl(wsId: string, lang = "en"): string {
  return `${API_BASE_URL}/v1/workspaces/${wsId}/leave/upload-template?lang=${lang}`;
}

/** Download the leave import template Excel via authenticated fetch. */
export async function downloadLeaveImportTemplate(wsId: string, lang = "en"): Promise<void> {
  const response = await apiFetch(`/v1/workspaces/${wsId}/leave/upload-template?lang=${lang}`);
  if (!response.ok) throw new Error("Failed to download template");
  downloadBlob(await response.blob(), "leave_import_template.xlsx");
}

/** Download the error Excel file via authenticated fetch. */
export async function downloadLeaveImportErrors(wsId: string, jobId: string): Promise<void> {
  const response = await apiFetch(`/v1/workspaces/${wsId}/leave/import/${jobId}/errors`);
  if (!response.ok) await ApiError.rejectResponse(response);
  downloadBlob(await response.blob(), `leave_import_errors_${jobId}.xlsx`);
}

// ── Leave Behalf Assignments (factory-generated CRUD) ─────────────────────

const leaveBehalfApi = createBehalfAssignmentApi({
  basePath: "/v1/workspaces/{ws_id}/leave/behalf-assignments",
});

export const fetchLeaveBehalfAssignments = (wsId: string) => leaveBehalfApi.list(wsId);
export const createLeaveBehalfAssignment = (wsId: string, data: Parameters<typeof leaveBehalfApi.create>[1]) => leaveBehalfApi.create(wsId, data);
export const deleteLeaveBehalfAssignment = (wsId: string, id: string) => leaveBehalfApi.delete(wsId, id);
