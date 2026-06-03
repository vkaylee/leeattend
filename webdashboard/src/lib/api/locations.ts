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
  workspace_id: string;
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

export async function fetchLocations(wsId: string): Promise<WorkLocation[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/locations", {
            params: { path: { ws_id: wsId } }
      });
  return data as WorkLocation[];
}

export async function createLocation(wsId: string, payload: CreateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await client.POST("/v1/workspaces/{ws_id}/locations", {
            params: { path: { ws_id: wsId } },
          body: payload });
  return data as WorkLocation;
}

export async function updateLocation(wsId: string, id: string, payload: UpdateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await client.PUT("/v1/workspaces/{ws_id}/locations/{id}", {
            params: { path: { ws_id: wsId, id } },
          body: payload });
  return data as WorkLocation;
}

export async function deleteLocation(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/locations/{id}", {
            params: { path: { ws_id: wsId, id } }
      });
}
