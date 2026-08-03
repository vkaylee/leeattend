import type { components } from "@/types/api-generated";

import { client } from "./client";

export type CreateStorageConfigRequest = components["schemas"]["CreateStorageConfigRequest"];

function throwIfError(error: unknown): never {
  if (error instanceof Error) throw error;
  throw new Error(String(error));
}

export async function fetchStorageConfig(wsId: string) {
  const { data, error } = await client.GET("/v1/workspaces/{workspace_id}/settings/storage", {
    params: { path: { workspace_id: wsId } },
  });
  if (error) throwIfError(error);
  return data;
}

export async function upsertStorageConfig(wsId: string, payload: CreateStorageConfigRequest) {
  const { data, error } = await client.PUT("/v1/workspaces/{workspace_id}/settings/storage", {
    params: { path: { workspace_id: wsId } },
    body: payload,
  });
  if (error) throwIfError(error);
  return data;
}

export async function testStorageConfig(wsId: string, payload: CreateStorageConfigRequest) {
  const { data, error } = await client.POST("/v1/workspaces/{workspace_id}/settings/storage/test", {
    params: { path: { workspace_id: wsId } },
    body: payload,
  });
  if (error) throwIfError(error);
  return data;
}

export async function deleteStorageConfig(wsId: string): Promise<null> {
  const { error } = await client.DELETE("/v1/workspaces/{workspace_id}/settings/storage", {
    params: { path: { workspace_id: wsId } },
  });
  if (error) throwIfError(error);
  // DELETE returns 204 No Content (void in generated types) — always null
  return null;
}
