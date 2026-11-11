import type { CreateEmployerRequest, Employer, UpdateEmployerRequest } from "./types";

import { client } from "./client";

export async function fetchEmployers(): Promise<Employer[]> {
  const { data } = await client.GET<{ data: Employer[] }>("/api/v1/employers");
  return data.data;
}

export async function createEmployer(
  data: CreateEmployerRequest,
): Promise<Employer> {
  const { data: resData } = await client.POST<Employer>("/api/v1/employers", {
    body: data,
  });
  return resData;
}

export async function updateEmployer(
  id: string,
  data: UpdateEmployerRequest,
): Promise<Employer> {
  const { data: resData } = await client.PUT<Employer>("/api/v1/employers/{id}", {
    params: { path: { id } },
    body: data,
  });
  return resData;
}

export async function deleteEmployer(id: string): Promise<void> {
  await client.DELETE("/api/v1/employers/{id}", {
    params: { path: { id } },
  });
}
