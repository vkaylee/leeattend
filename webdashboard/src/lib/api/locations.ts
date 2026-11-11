import { unwrapDataArray } from "@/types/api-overrides";

import { client } from "./client";

export interface WorkLocation {
  address?: string;
  created_at: string;
  id: string;
  is_headquarters: boolean;
  name: string;
  status: string;
  timezone: string;
  updated_at: string;
}

export interface CreateWorkLocationRequest {
  address?: string;
  is_headquarters?: boolean;
  name: string;
  status?: string;
  timezone?: string;
}

export interface UpdateWorkLocationRequest {
  address?: string;
  is_headquarters?: boolean;
  name?: string;
  status?: string;
  timezone?: string;
}

export async function fetchLocations(): Promise<WorkLocation[]> {
  const { data } = await client.GET<WorkLocation[]>("/api/v1/locations");
  return unwrapDataArray<WorkLocation>(data);
}

export async function createLocation(payload: CreateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await client.POST<WorkLocation>("/api/v1/locations", { body: payload });
  return data;
}

export async function updateLocation(id: string, payload: UpdateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await client.PUT<WorkLocation>("/api/v1/locations/{id}", {
    params: { path: { id } },
    body: payload,
  });
  return data;
}

export async function deleteLocation(id: string): Promise<void> {
  await client.DELETE("/api/v1/locations/{id}", {
    params: { path: { id } },
  });
}
