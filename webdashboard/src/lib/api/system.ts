import { client } from "./client";

export async function fetchSystemHealth(): Promise<unknown> {
  const { data: resData } = await client.GET("/system/health");
  return resData;
}
