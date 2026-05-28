import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Department } from "./types";
import { apiClient } from "./client";

export async function fetchDepartments(wsId: string): Promise<Department[]> {
  const { data: resData } = await apiClient.get<Department[]>(`/v1/workspaces/${wsId}/departments`);
    return resData;
}

export async function createDepartment(wsId: string, data: Partial<Department>): Promise<Department> {
  const { data: resData } = await apiClient.post<Department>(`/v1/workspaces/${wsId}/departments`, data);
    return resData;
}

export async function updateDepartment(wsId: string, id: string, data: Partial<Department>): Promise<Department> {
  const { data: resData } = await apiClient.patch<Department>(`/v1/workspaces/${wsId}/departments/${id}`, data);
    return resData;
}

export async function deleteDepartment(wsId: string, id: string): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/departments/${id}`);
}
