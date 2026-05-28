import { apiClient } from "./client";

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
  const { data } = await apiClient.get<WorkLocation[]>(`/workspaces/${wsId}/locations`);
  return data;
}

export async function createLocation(wsId: string, payload: CreateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await apiClient.post<WorkLocation>(`/workspaces/${wsId}/locations`, payload);
  return data;
}

export async function updateLocation(wsId: string, id: string, payload: UpdateWorkLocationRequest): Promise<WorkLocation> {
  const { data } = await apiClient.put<WorkLocation>(`/workspaces/${wsId}/locations/${id}`, payload);
  return data;
}

export async function deleteLocation(wsId: string, id: string): Promise<void> {
  await apiClient.delete(`/workspaces/${wsId}/locations/${id}`);
}
