import { client } from "./client";
import { Department } from "./types";

export async function fetchDepartments(wsId: string): Promise<Department[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/departments", {
            params: { path: { ws_id: wsId } }
      });
    return resData!;
}

export async function createDepartment(
  wsId: string,
  data: { name: string; description?: string | null; parent_id?: string | null; manager_id?: string | null; default_working_pattern_id?: string | null }
): Promise<Department> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/departments", {
            params: { path: { ws_id: wsId } },
          body: data });
    return resData!;
}

export async function updateDepartment(
  wsId: string,
  id: string,
  data: { name?: string; description?: string | null; parent_id?: string | null; manager_id?: string | null; default_working_pattern_id?: string | null }
): Promise<Department> {
  const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/departments/{id}", {
            params: { path: { ws_id: wsId, id } },
          body: data });
    return resData!;
}

export async function deleteDepartment(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/departments/{id}", {
            params: { path: { ws_id: wsId, id } }
      });
}
