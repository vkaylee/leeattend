import { components } from "@/types/api-generated";

import { client } from "./client";

export type CreateStorageConfigRequest = components["schemas"]["CreateStorageConfigRequest"];

export async function fetchStorageConfig(wsId: string) {
  const { data, error } = await client.GET("/v1/workspaces/{workspace_id}/settings/storage", {
    params: {
      path: { workspace_id: wsId },
    },
  });
  if (error) throw error;
  return data;
}

export async function upsertStorageConfig(wsId: string, payload: CreateStorageConfigRequest) {
  const { data, error } = await client.PUT("/v1/workspaces/{workspace_id}/settings/storage", {
    params: {
      path: { workspace_id: wsId },
    },
    body: payload,
  });
  if (error) throw error;
  return data;
}

export async function testStorageConfig(wsId: string, payload: CreateStorageConfigRequest) {
  const { data, error } = await client.POST("/v1/workspaces/{workspace_id}/settings/storage/test", {
    params: {
      path: { workspace_id: wsId },
    },
    body: payload,
  });
  if (error) throw error;
  return data;
}

export async function deleteStorageConfig(wsId: string) {
  const { data, error } = await client.DELETE("/v1/workspaces/{workspace_id}/settings/storage", {
    params: {
      path: { workspace_id: wsId },
    },
  });
  if (error) throw error;
  return data ?? null;
}
