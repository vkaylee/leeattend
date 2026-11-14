import type { Department } from "./types";

import { client } from "./client";

export async function fetchDepartments(): Promise<Department[]> {
  const { data } = await client.GET<{ data: Department[] }>("/api/v1/departments");
  return data.data;
}

export async function createDepartment(
  data: { name: string; description?: string | null; parent_id?: string | null; manager_id?: string | null },
): Promise<Department> {
  const { data: result } = await client.POST<Department>("/api/v1/departments", { body: data });
  return result;
}

export async function updateDepartment(
  id: string,
  data: { name?: string; description?: string | null; parent_id?: string | null; manager_id?: string | null },
): Promise<Department> {
  const { data: result } = await client.PATCH<Department>(`/api/v1/departments/${encodeURIComponent(id)}`, { body: data });
  return result;
}

export async function deleteDepartment(id: string): Promise<void> {
  await client.DELETE(`/api/v1/departments/${encodeURIComponent(id)}`);
}
