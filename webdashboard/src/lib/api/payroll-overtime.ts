import { client, fetch } from "./client";

export type WorkPolicyVersionStatus = "draft" | "published" | "archived";
export type OvertimeDayType = "weekday" | "weekend" | "holiday" | "any";

export interface OvertimeRuleV2 {
  day_type: OvertimeDayType;
  days_of_week: number;
  multiplier: string;
  priority: number;
  time_end?: string | null;
  time_start?: string | null;
}

export interface OvertimePolicyV2 {
  created_at: string;
  description?: string | null;
  id: string;
  latest_published_version_id?: string | null;
  latest_published_version_no?: number | null;
  name: string;
  updated_at: string;
}

export interface OvertimePolicyVersion {
  created_at: string;
  created_by: string;
  daily_cap_minutes?: number | null;
  id: string;
  monthly_cap_hours?: string | null;
  policy_id: string;
  published_at?: string | null;
  published_by?: string | null;
  row_version: number;
  rules: OvertimeRuleV2[];
  status: WorkPolicyVersionStatus;
  updated_at: string;
  version_no: number;
  yearly_cap_hours?: string | null;
}

export interface OvertimePolicyDetail {
  policy: OvertimePolicyV2;
  versions: OvertimePolicyVersion[];
}

export async function fetchOvertimePolicies(): Promise<OvertimePolicyV2[]> {
  const { data: resData } = await client.GET<{ data: OvertimePolicyV2[] }>("/api/v1/overtime/policies");
  return resData?.data ?? [];
}

export async function fetchOvertimePolicy(policyId: string): Promise<OvertimePolicyDetail> {
  const { data: resData } = await fetch<OvertimePolicyDetail>("GET", "/api/v1/overtime/policies/{policy_id}", {
    params: { path: { policy_id: policyId } },
  });
  return resData;
}

export async function createOvertimePolicy(body: {
  daily_cap_minutes?: number | null;
  description?: string | null;
  monthly_cap_hours?: string | null;
  name: string;
  rules?: OvertimeRuleV2[];
  yearly_cap_hours?: string | null;
}): Promise<OvertimePolicyDetail> {
  const { data: resData } = await fetch<OvertimePolicyDetail>("POST", "/api/v1/overtime/policies", { body });
  return resData;
}

export async function createOvertimePolicyVersion(
  policyId: string,
  body: {
    daily_cap_minutes?: number | null;
    monthly_cap_hours?: string | null;
    rules: OvertimeRuleV2[];
    yearly_cap_hours?: string | null;
  },
): Promise<OvertimePolicyVersion> {
  const { data } = await fetch<OvertimePolicyVersion>("POST", "/api/v1/overtime/policies/{policy_id}/versions", {
    params: { path: { policy_id: policyId } },
    body,
  });
  return data;
}

export async function patchOvertimePolicyVersion(
  policyId: string,
  versionId: string,
  body: {
    daily_cap_minutes?: number | null;
    expected_row_version: number;
    monthly_cap_hours?: string | null;
    rules: OvertimeRuleV2[];
    yearly_cap_hours?: string | null;
  },
): Promise<OvertimePolicyVersion> {
  const { data } = await fetch<OvertimePolicyVersion>("PATCH", "/api/v1/overtime/policies/{policy_id}/versions/{version_id}", {
    params: { path: { policy_id: policyId, version_id: versionId } },
    body,
  });
  return data;
}

export async function publishOvertimePolicyVersion(
  policyId: string,
  versionId: string,
  expectedRowVersion: number,
): Promise<OvertimePolicyVersion> {
  const { data } = await fetch<OvertimePolicyVersion>("POST", "/api/v1/overtime/policies/{policy_id}/versions/{version_id}/publish", {
    params: { path: { policy_id: policyId, version_id: versionId } },
    body: { expected_row_version: expectedRowVersion },
  });
  return data;
}
