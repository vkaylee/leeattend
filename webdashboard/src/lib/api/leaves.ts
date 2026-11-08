/* eslint-disable max-lines -- API wrapper: 400+ lines of thin exported functions; splitting would scatter imports */
import type { components } from "@/types/api-generated";

import type { LeavePolicyAssignmentDetail, LeaveRequest, EmployeeLeaveBalance, WorkspaceLeaveBalance, LeaveHistoryItem, LeavePolicy, LeaveType, LeaveTypeConfig, PaginatedResponse } from './types';

import { createBehalfAssignmentApi } from "./behalf-assignment-factory";
import { client } from "./client";
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
  data: SubmitLeaveRequestBody,
): Promise<LeaveRequest> {
  const { data: resData } = await client.POST<LeaveRequest>("/v1/leave/submit", {
    body: data,
  });
  return resData;
}

export async function fetchWorkspaceLeaveRequests(
  params?: {
    status?: "pending" | "approved" | "rejected" | "cancelled" | "pre_approved";
    employee_id?: string;
    page?: number;
    limit?: number;
  }
): Promise<PaginatedResponse<LeaveRequest>> {
  const { data: resData } = await client.GET<PaginatedResponse<LeaveRequest>>("/v1/leave/requests", {
    params: {
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
  const { data: resData } = await client.GET<LeaveRequest[]>("/v1/employees/{id}/leave/requests", {
    params: {
      path: { id: employeeId },
    },
  });
  return resData;
}

export async function approveLeaveRequest(
  requestId: string,
  data: ApproveLeaveRequestBody,
): Promise<void> {
  await client.POST("/v1/leave/requests/{id}/approve", {
    params: {
      path: { id: requestId },
    },
    body: data,
  });
}

export async function fetchEmployeeLeaveBalances(
  employeeId: string,
): Promise<EmployeeLeaveBalance[]> {
  const { data: resData } = await client.GET<EmployeeLeaveBalance[]>("/v1/employees/{id}/leave/balances", {
    params: {
      path: { id: employeeId },
    },
  });
  return resData;
}

export async function fetchWorkspaceLeaveBalances(
  params?: { page?: number; limit?: number; q?: string; year?: number }
): Promise<PaginatedResponse<WorkspaceLeaveBalance>> {
  const { data: resData } = await client.GET<PaginatedResponse<WorkspaceLeaveBalance>>("/v1/leave/balances", {
    params: {
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
  employeeId: string,
  leaveTypeId?: string,
): Promise<LeaveHistoryItem[]> {
  const { data: resData } = await client.GET<LeaveHistoryItem[]>("/v1/employees/{id}/leave/history", {
    params: {
      path: { id: employeeId },
      query: leaveTypeId ? { leave_type_id: leaveTypeId } : undefined,
    },
  });
  return resData;
}

export async function adjustLeaveBalance(
  employeeId: string,
  data: AdjustBalanceRequestBody,
): Promise<void> {
  await client.POST("/v1/employees/{id}/leave/adjust", {
    params: {
      path: { id: employeeId },
    },
    body: data,
  });
}

export async function fetchLeavePolicies(): Promise<LeavePolicy[]> {
  const { data: resData } = await client.GET<LeavePolicy[]>("/v1/leave/policies");
  return resData;
}

export async function createLeavePolicy(
  data: CreateLeavePolicyRequestBody
): Promise<LeavePolicy> {
  const { data: resData } = await client.POST<LeavePolicy>("/v1/leave/policies", {
    body: data,
  });
  return resData;
}

export async function updateLeavePolicy(
  policyId: string,
  data: UpdateLeavePolicyRequestBody
): Promise<LeavePolicy> {
  const { data: resData } = await client.PATCH<LeavePolicy>("/v1/leave/policies/{policy_id}", {
    params: { path: { policy_id: policyId } },
    body: data,
  });
  return resData;
}

export async function deleteLeavePolicy(
  policyId: string
): Promise<void> {
  await client.DELETE("/v1/leave/policies/{policy_id}", {
    params: {
      path: { policy_id: policyId },
    },
  });
}

export async function fetchLeaveTypes(): Promise<LeaveType[]> {
  const { data: resData } = await client.GET<LeaveType[]>("/v1/leave/types");
  return resData;
}

export async function createLeaveType(
  data: CreateLeaveTypeRequestBody
): Promise<LeaveType> {
  const { data: resData } = await client.POST<LeaveType>("/v1/leave/types", {
    body: data,
  });
  return resData;
}

export async function deleteLeaveType(
  id: string
): Promise<void> {
  await client.DELETE("/v1/leave/types/{id}", {
    params: {
      path: { id },
    },
  });
}

export async function restoreLeaveType(
  id: string
): Promise<LeaveType> {
  const { data: resData } = await client.POST<LeaveType>(
    "/v1/leave/types/{id}/restore",
    {
      params: {
        path: { id },
      },
    }
  );
  return resData;
}

export async function fetchLeavePolicyAssignments(): Promise<LeavePolicyAssignmentDetail[]> {
  const { data: resData } = await client.GET<LeavePolicyAssignmentDetail[]>("/v1/leave/policies/assignments");
  return resData;
}

export async function assignLeavePolicy(
  data: AssignLeavePolicyRequestBody
): Promise<void> {
  await client.POST("/v1/leave/policies/assign", {
    body: data,
  });
}

export async function deleteLeavePolicyAssignment(
  assignmentId: string
): Promise<void> {
  await client.DELETE("/v1/leave/policies/assignments/{assignment_id}", {
    params: {
      path: { assignment_id: assignmentId },
    },
  });
}

export interface AccrualJobResponse {
  job_id: string;
  message: string;
  status: string;
}

export async function triggerAccrualRun(
  data: TriggerAccrualRequestBody
): Promise<AccrualJobResponse> {
  const { data: resData } = await client.POST<AccrualJobResponse>("/v1/leave/accrual/run", {
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
  jobId: string
): Promise<JobStatusResponse> {
  const { data: resData } = await client.GET<JobStatusResponse>("/v1/jobs/{job_id}/status", {
    params: { path: { job_id: jobId } },
  });
  return resData;
}

export async function updateLeaveTypeConfig(
  id: string,
  config: LeaveTypeConfig
): Promise<LeaveType> {
  const { data: resData } = await client.PUT<LeaveType>("/v1/leave/types/{id}/config", {
    params: {
      path: { id },
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
  status: 'pending' | 'processing' | 'partially_failed' | 'completed' | 'failed';
  total_rows: number;
  updated_at: string;
}

/** Upload an Excel file for bulk leave request import. */
export async function uploadLeaveExcel(
  file: File,
): Promise<{ job_id: string; status: string }> {
  const formData = new FormData();
  formData.append('file', file);
  const { data: resData } = await client.POST<{ job_id: string; status: string }>("/v1/leave/upload-excel", {
    body: formData,
  });
  return resData;
}

/** Fetch the latest leave import job for the workspace (page resume). */
export async function fetchLatestLeaveImport(): Promise<LeaveImportJob | null> {
  const { data: resData } = await client.GET<LeaveImportJob | null>("/v1/leave/import/latest");
  return resData;
}

/** Fetch the latest leave import job (for polling). */
export async function fetchLeaveImportStatus(): Promise<LeaveImportJob | null> {
  const { data: resData } = await client.GET<LeaveImportJob | null>("/v1/leave/import/latest");
  return resData;
}

/** Get the URL for downloading the error Excel file (URL-only, for non-fetch usage). */
export function getLeaveImportErrorsUrl(jobId: string): string {
  return `${API_BASE_URL}/v1/leave/import/${jobId}/errors`;
}

/** Get the URL for downloading the import template Excel (URL-only, for non-fetch usage). */
export function getLeaveImportTemplateUrl(lang = "en"): string {
  return `${API_BASE_URL}/v1/leave/upload-template?lang=${lang}`;
}

/** Download the leave import template Excel via authenticated fetch. */
export async function downloadLeaveImportTemplate(lang = "en"): Promise<void> {
  const response = await apiFetch(`/v1/leave/upload-template?lang=${lang}`);
  if (!response.ok) throw new Error("Failed to download template");
  downloadBlob(await response.blob(), "leave_import_template.xlsx");
}

/** Download the error Excel file via authenticated fetch. */
export async function downloadLeaveImportErrors(jobId: string): Promise<void> {
  const response = await apiFetch(`/v1/leave/import/${jobId}/errors`);
  if (!response.ok) await ApiError.rejectResponse(response);
  downloadBlob(await response.blob(), `leave_import_errors_${jobId}.xlsx`);
}

// ── Leave Behalf Assignments (factory-generated CRUD) ─────────────────────

const leaveBehalfApi = createBehalfAssignmentApi({
  basePath: "/v1/leave/behalf-assignments",
});

export const fetchLeaveBehalfAssignments = () => leaveBehalfApi.list();
export const createLeaveBehalfAssignment = (data: Parameters<typeof leaveBehalfApi.create>[0]) => leaveBehalfApi.create(data);
export const deleteLeaveBehalfAssignment = (id: string) => leaveBehalfApi.delete(id);
