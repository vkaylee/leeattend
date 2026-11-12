import { client } from "./client";

type RoundingPolicy = "none" | "floor" | "round" | "ceil";

export interface AttendanceRules {
  early_arrival_strategy?: "allow" | "block" | "clamp" | null;
  grace_period_minutes?: number | null;
  rounding_policy: RoundingPolicy;
  work_unit_minutes: number;
}

export interface AttendancePolicyVersion {
  created_at: string;
  created_by?: string | null;
  early_arrival_strategy: "allow" | "block" | "clamp";
  grace_period_minutes: number;
  id: string;
  rounding_policy: RoundingPolicy;
  valid_from: string;
  valid_to?: string | null;
  work_unit_minutes: number;
}

export interface UpdateAttendanceRulesRequest {
  early_arrival_strategy?: "allow" | "block" | "clamp";
  grace_period_minutes?: number;
  rounding_policy?: RoundingPolicy;
  valid_from?: string;
  work_unit_minutes?: number;
}

export async function fetchDefaultAttendanceRules(): Promise<AttendanceRules> {
  const { data: resData } = await client.GET<AttendanceRules>("/api/v1/attendance/rules");
  return resData;
}

export async function fetchAttendancePolicyVersions(): Promise<AttendancePolicyVersion[]> {
  const { data: resData } = await client.GET<AttendancePolicyVersion[]>("/api/v1/attendance/rules/versions");
  return resData;
}

export interface PolicyAssignmentResponse {
  id: string;
  rules: AttendanceRules;
  target_id: string;
  target_type: string;
  target_name?: string | null;
  target_code?: string | null;
  department_name?: string | null;
  valid_from: string;
  valid_to?: string | null;
}

export interface PolicyAssignmentListResponse {
  data: PolicyAssignmentResponse[];
  total: number;
  page: number;
  limit: number;
}

export interface PolicyAssignmentListParams {
  limit?: number;
  page?: number;
  q?: string;
  target_type?: "USER" | "GROUP";
}

export interface UpsertPolicyAssignmentRequest {
  early_arrival_strategy: "allow" | "block" | "clamp";
  grace_period_minutes?: number | null;
  rounding_policy: RoundingPolicy;
  target_id: string;
  target_type: "USER" | "GROUP";
  valid_from?: string;
  work_unit_minutes: number;
}

export async function fetchPolicyAssignments(
  params: PolicyAssignmentListParams = {},
): Promise<PolicyAssignmentListResponse> {
  const { data: resData } = await client.GET<PolicyAssignmentListResponse>("/api/v1/attendance/policies/assignments", {
    params: { query: { ...params } as Record<string, string | number | undefined> },
  });
  return resData;
}

export async function upsertPolicyAssignment(
  data: UpsertPolicyAssignmentRequest
): Promise<PolicyAssignmentResponse> {
  const { data: resData } = await client.POST<PolicyAssignmentResponse>("/api/v1/attendance/policies/assignments", {
    body: data,
  });
  return resData;
}

export async function deletePolicyAssignment(assignmentId: string): Promise<void> {
  await client.DELETE("/api/v1/attendance/policies/assignments/{assignment_id}", {
    params: { path: { assignment_id: assignmentId } },
  });
}

export async function updateDefaultAttendanceRules(
  data: UpdateAttendanceRulesRequest
): Promise<AttendancePolicyVersion> {
  const { data: resData } = await client.PATCH<AttendancePolicyVersion>("/api/v1/attendance/rules", {
    body: data,
  });
  return resData;
}
