import { client } from "./client";

export interface AttendanceRules {
  early_arrival_strategy?: "allow" | "block" | "clamp" | null;
  grace_period_minutes?: number | null;
  rounding_policy: string;
  work_unit_minutes: number;
}

export interface AttendancePolicyVersion {
  created_at: string;
  created_by?: string | null;
  early_arrival_strategy: "allow" | "block" | "clamp";
  grace_period_minutes: number;
  id: string;
  rounding_policy: string;
  valid_from: string;
  valid_to?: string | null;
  work_unit_minutes: number;
}

export interface UpdateAttendanceRulesRequest {
  early_arrival_strategy?: "allow" | "block" | "clamp";
  grace_period_minutes?: number;
  rounding_policy?: string;
  valid_from?: string;
  work_unit_minutes?: number;
}

export async function fetchDefaultAttendanceRules(wsId: string): Promise<AttendanceRules> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/rules", {
    params: { path: { ws_id: wsId } }
  });
  return resData;
}

export async function fetchAttendancePolicyVersions(wsId: string): Promise<AttendancePolicyVersion[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/rules/versions", {
    params: { path: { ws_id: wsId } }
  });
  return resData;
}

export interface PolicyAssignmentResponse {
  id: string;
  rules: AttendanceRules;
  target_id: string;
  target_type: string;
  valid_from: string;
  valid_to?: string | null;
}

export interface UpsertPolicyAssignmentRequest {
  early_arrival_strategy: "allow" | "block" | "clamp";
  grace_period_minutes?: number | null;
  rounding_policy: string;
  target_id: string;
  target_type: "USER" | "GROUP";
  valid_from?: string;
  work_unit_minutes: number;
}

export async function fetchPolicyAssignments(wsId: string): Promise<PolicyAssignmentResponse[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/policies/assignments", {
    params: { path: { ws_id: wsId } }
  });
  return resData;
}

export async function upsertPolicyAssignment(
  wsId: string,
  data: UpsertPolicyAssignmentRequest
): Promise<PolicyAssignmentResponse> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/policies/assignments", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  return resData;
}

export async function deletePolicyAssignment(wsId: string, assignmentId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}", {
    params: { path: { ws_id: wsId, assignment_id: assignmentId } }
  });
}

export async function updateDefaultAttendanceRules(
  wsId: string,
  data: UpdateAttendanceRulesRequest
): Promise<AttendancePolicyVersion> {
  const { data: resData } = await client.PATCH("/v1/workspaces/{ws_id}/attendance/rules", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  return resData;
}
