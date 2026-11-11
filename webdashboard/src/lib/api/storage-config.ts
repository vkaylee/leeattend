import type { components } from "@/types/api-generated";

import { client } from "./client";

export type CreateStorageConfigRequest = components["schemas"]["CreateStorageConfigRequest"];
export type StorageConfigResponse = components["schemas"]["StorageConfigResponse"];

export async function fetchStorageConfig(): Promise<StorageConfigResponse> {
  const { data } = await client.GET<StorageConfigResponse>("/api/v1/settings/storage");
  return data;
}

export async function upsertStorageConfig(
  payload: CreateStorageConfigRequest,
): Promise<StorageConfigResponse> {
  const { data } = await client.PUT<StorageConfigResponse>("/api/v1/settings/storage", {
    body: payload,
  });
  return data;
}

export async function testStorageConfig(
  payload: CreateStorageConfigRequest,
): Promise<boolean> {
  const { data } = await client.POST<boolean>("/api/v1/settings/storage/test", {
    body: payload,
  });
  return data;
}

export async function deleteStorageConfig(): Promise<null> {
  await client.DELETE("/api/v1/settings/storage");
  return null;
}
