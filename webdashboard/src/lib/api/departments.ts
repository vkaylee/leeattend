import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Department } from "./types";

export async function fetchDepartments(wsId: string): Promise<Department[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments`);
  if (!response.ok) throw new Error("Failed to fetch departments");
  return response.json();
}

export async function createDepartment(wsId: string, data: Partial<Department>): Promise<Department> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create department");
  return response.json();
}

export async function updateDepartment(wsId: string, id: string, data: Partial<Department>): Promise<Department> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${id}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update department");
  return response.json();
}

export async function deleteDepartment(wsId: string, id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete department");
}
