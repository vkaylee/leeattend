// @ts-nocheck
import { client } from "./client";

export interface WorkLocation {
  id: string;
  workspace_id: string;
  name: string;
  address?: string;
  timezone: string;
  is_headquarters: boolean;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface CreateWorkLocationRequest {
  name: string;
  address?: string;
  timezone?: string;
  is_headquarters?: boolean;
  status?: string;
}

export interface UpdateWorkLocationRequest {
  name?: string;
  address?: string;
  timezone?: string;
  is_headquarters?: boolean;
  status?: string;
}

export async function fetchLocations(wsId: string): Promise<WorkLocation[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/locations", {
            params: { path: { ws_id: wsId } }
      });
  return data!;
}

export async function createLocation(wsId: string, payload: CreateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await client.POST("/v1/workspaces/{ws_id}/locations", {
            params: { path: { ws_id: wsId } },
          body: payload });
  return data!;
}

export async function updateLocation(wsId: string, id: string, payload: UpdateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await client.PUT("/v1/workspaces/{ws_id}/locations/{id}", {
            params: { path: { ws_id: wsId, id } },
          body: payload });
  return data!;
}

export async function deleteLocation(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/locations/{id}", {
            params: { path: { ws_id: wsId, id } }
      });
}
