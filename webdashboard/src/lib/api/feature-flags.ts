import { fetch as apiFetch, client } from "./client";

export interface FeatureFlag {
  id: string;
  flag_key: string;
  enabled: boolean;
  description: string | null;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface EffectiveFeatureFlag {
  key: string;
  enabled: boolean;
}

export async function fetchFeatureFlags(): Promise<FeatureFlag[]> {
  const { data } = await client.GET<FeatureFlag[]>("/api/v1/feature-flags");
  return data;
}

export async function fetchEffectiveFeatureFlags(): Promise<EffectiveFeatureFlag[]> {
  const { data } = await client.GET<EffectiveFeatureFlag[]>("/api/v1/feature-flags/effective");
  return data;
}

export async function updateFeatureFlag(id: string, enabled: boolean): Promise<FeatureFlag> {
  const { data } = await client.PATCH<FeatureFlag>(`/api/v1/feature-flags/${id}`, { body: { enabled } });
  return data;
}

export async function upsertFeatureFlag(input: {
  flag_key: string;
  enabled: boolean;
  description?: string;
}): Promise<FeatureFlag> {
  const { data } = await client.PUT<FeatureFlag>("/api/v1/feature-flags", { body: input });
  return data;
}

export async function checkFeatureFlag(key: string): Promise<EffectiveFeatureFlag> {
  const { data } = await apiFetch<EffectiveFeatureFlag>("GET", `/api/v1/feature-flags/check/${encodeURIComponent(key)}`);
  return data;
}
