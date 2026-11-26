import type {
  CreateWorkPolicyGroupRequest,
  CreateWorkPolicyVersionRequest,
  PatchWorkPolicyVersionRequest,
  PublishWorkPolicyVersionRequest,
  ResolvedWorkPolicyResponse,
  TransferWorkPolicyMembershipResponse,
  WorkPolicyBatchSimulationResponse,
  WorkPolicyGroupSummary,
  WorkPolicyMembership,
  WorkPolicyValidationResponse,
  WorkPolicyVersion,
} from "./payroll-work-policy-types";
import type { WorkPolicyVersionStatus } from "./payroll-overtime";

import { fetch } from "./client";
import { fetchOvertimePolicies } from "./payroll-overtime";
import { fetchSalaryComponents } from "./payroll-runs";
import { fetchWorkingPatterns } from "./working-patterns";

interface PaginatedResponse<T> {
  data: T[];
  limit: number;
  page: number;
  total: number;
}

export async function fetchWorkPolicyReferenceOptions(): Promise<{
  overtimeVersions: { id: string; label: string }[];
  patterns: { id: string; label: string }[];
  salaryComponents: { id: string; label: string }[];
}> {
  const [patterns, overtime, salaryComponents] = await Promise.all([
    fetchWorkingPatterns(),
    fetchOvertimePolicies(),
    fetchSalaryComponents(),
  ]);
  const overtimeVersions = overtime.flatMap((item) =>
    item.latest_published_version_id
      ? [{ id: item.latest_published_version_id, label: `${item.name} (v${item.latest_published_version_no ?? 1})` }]
      : [],
  );
  return {
    overtimeVersions,
    patterns: patterns.map((item) => ({ id: item.id, label: item.name })),
    salaryComponents: salaryComponents.map((item) => ({ id: item.id, label: `${item.name} (${item.code})` })),
  };
}

export async function fetchWorkPolicyGroups(): Promise<WorkPolicyGroupSummary[]> {
  const { data } = await fetch<PaginatedResponse<WorkPolicyGroupSummary>>("GET", "/api/v1/payroll/work-policy-groups", {
    params: { query: { page: 1, limit: 100 } },
  });
  return data?.data ?? [];
}

export async function createWorkPolicyGroup(body: CreateWorkPolicyGroupRequest): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("POST", "/api/v1/payroll/work-policy-groups", { body });
  return data;
}

export async function fetchWorkPolicyGroupVersions(groupId: string, status?: WorkPolicyVersionStatus): Promise<WorkPolicyVersion[]> {
  const { data } = await fetch<PaginatedResponse<WorkPolicyVersion>>("GET", "/api/v1/payroll/work-policy-groups/{group_id}/versions", {
    params: { path: { group_id: groupId }, query: { page: 1, limit: 100, ...(status ? { status } : {}) } },
  });
  return data?.data ?? [];
}

export async function createWorkPolicyVersion(groupId: string, body: CreateWorkPolicyVersionRequest): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions", {
    params: { path: { group_id: groupId } },
    body,
  });
  return data;
}

export async function patchWorkPolicyVersion(groupId: string, versionId: string, body: PatchWorkPolicyVersionRequest): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("PATCH", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}", {
    params: { path: { group_id: groupId, version_id: versionId } },
    body,
  });
  return data;
}

export async function validateWorkPolicyVersion(groupId: string, versionId: string): Promise<WorkPolicyValidationResponse> {
  const { data } = await fetch<WorkPolicyValidationResponse>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/validate", {
    params: { path: { group_id: groupId, version_id: versionId } },
  });
  return data;
}

export async function publishWorkPolicyVersion(
  groupId: string,
  versionId: string,
  body: PublishWorkPolicyVersionRequest,
  idempotencyKey?: string,
): Promise<WorkPolicyVersion> {
  const { data } = await fetch<WorkPolicyVersion>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/publish", {
    params: { path: { group_id: groupId, version_id: versionId } },
    headers: idempotencyKey ? { "Idempotency-Key": idempotencyKey } : undefined,
    body,
  });
  return data;
}

export async function fetchWorkPolicyMemberships(groupId: string): Promise<WorkPolicyMembership[]> {
  const { data } = await fetch<WorkPolicyMembership[]>("GET", "/api/v1/payroll/work-policy-groups/{group_id}/memberships", {
    params: { path: { group_id: groupId } },
  });
  return data ?? [];
}

export async function createWorkPolicyMembership(
  groupId: string,
  body: { effective_from: string; effective_to?: string | null; employment_relationship_id: string },
): Promise<WorkPolicyMembership> {
  const { data } = await fetch<WorkPolicyMembership>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/memberships", {
    params: { path: { group_id: groupId } },
    body,
  });
  return data;
}

export async function transferWorkPolicyMembership(body: {
  effective_date: string;
  employment_relationship_id: string;
  target_group_id: string;
}): Promise<TransferWorkPolicyMembershipResponse> {
  const { data } = await fetch<TransferWorkPolicyMembershipResponse>("POST", "/api/v1/payroll/work-policy-memberships/transfer", { body });
  return data;
}

export async function resolveWorkPolicyByRelationship(relationshipId: string, asOf: string): Promise<ResolvedWorkPolicyResponse> {
  const { data } = await fetch<ResolvedWorkPolicyResponse>("GET", "/api/v1/employment-relationships/{relationship_id}/work-policy", {
    params: { path: { relationship_id: relationshipId }, query: { as_of: asOf } },
  });
  return data;
}

export async function simulateWorkPolicyVersionBatch(
  groupId: string,
  versionId: string,
  body: { employment_relationship_ids: string[]; month: number; year: number },
): Promise<WorkPolicyBatchSimulationResponse> {
  const { data } = await fetch<WorkPolicyBatchSimulationResponse>("POST", "/api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/simulate", {
    params: { path: { group_id: groupId, version_id: versionId } },
    body,
  });
  return data;
}
