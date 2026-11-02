import type { components } from "@/types/api-generated";

import { client } from "./client";
import { Device } from "./types";

// ── Typed request body aliases (from generated OpenAPI schema) ──────────────

export type CreateDeviceRequest = components["schemas"]["CreateDeviceRequest"];
export type UpdateDeviceRequest = components["schemas"]["UpdateDeviceRequest"];


export async function fetchDevices(): Promise<Device[]> {
  const { data: resData } = await client.GET("/devices");
  return resData;
}

export async function createDevice(device: CreateDeviceRequest): Promise<{ id: string }> {
  const { data: resData } = await client.POST("/devices", {
    body: device,
  });
  return resData;
}

export async function updateDevice(id: string, device: UpdateDeviceRequest): Promise<void> {
  await client.PUT("/devices/{id}", {
    params: { path: { id } },
    body: device,
  });
}

export async function deleteDevice(id: string): Promise<void> {
  await client.DELETE("/devices/{id}", {
    params: { path: { id } },
  });
}



