import { client } from "./client";
import { Employer, CreateEmployerRequest, UpdateEmployerRequest } from "./types";

export async function fetchEmployers(wsId: string): Promise<Employer[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/employers", {
    params: { path: { ws_id: wsId } }
  });
  return resData as Employer[];
}

export async function createEmployer(
  wsId: string,
  data: CreateEmployerRequest
): Promise<Employer> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/employers", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  return resData as Employer;
}

export async function updateEmployer(
  wsId: string,
  id: string,
  data: UpdateEmployerRequest
): Promise<Employer> {
  const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/employers/{id}", {
    params: { path: { ws_id: wsId, id } },
    body: data
  });
  return resData as Employer;
}

export async function deleteEmployer(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/employers/{id}", {
    params: { path: { ws_id: wsId, id } }
  });
}
