import type { components } from "@/types/api-generated";

import { client } from "./client";
import { AgentStatus, Device, AuthorizedAgent, DeviceInfo } from "./types";

// ── Typed request body aliases (from generated OpenAPI schema) ──────────────

export type CreateDeviceRequest = components["schemas"]["CreateDeviceRequest"];
export type UpdateDeviceRequest = components["schemas"]["UpdateDeviceRequest"];
export type CreateAuthorizedAgentRequest = components["schemas"]["CreateAuthorizedAgentRequest"];
export type UpdateAuthorizedAgentRequest = components["schemas"]["UpdateAuthorizedAgentRequest"];
type TrustAgentRequestBody = components["schemas"]["TrustAgentRequest"];

export async function fetchAgents(): Promise<AgentStatus[]> {
  const { data: resData } = await client.GET("/agents");
  return resData;
}

/** Send a command to a connected agent. Body type is `unknown` per generated schema. */
export async function sendAgentCommand(agentId: string, command: unknown): Promise<void> {
  await client.POST("/agent/{agent_id}/command", {
    params: { path: { agent_id: agentId } },
    body: command,
  });
}

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

export async function fetchAuthorizedAgents(includeInactive = false): Promise<AuthorizedAgent[]> {
  const { data: resData } = await client.GET("/authorized-agents", {
    params: {
      query: { include_inactive: includeInactive },
    },
  });
  return resData;
}

export async function fetchAuthorizedAgentById(id: string): Promise<AuthorizedAgent> {
  const { data: resData } = await client.GET("/authorized-agents/{id}", {
    params: { path: { id } },
  });
  return resData;
}

export async function createAuthorizedAgent(agent: CreateAuthorizedAgentRequest): Promise<{ id: string }> {
  const { data: resData } = await client.POST("/authorized-agents", {
    body: agent,
  });
  return resData;
}

export async function updateAgentTrust(id: string, is_trusted: boolean): Promise<void> {
  const body: TrustAgentRequestBody = { is_trusted };
  await client.PUT("/authorized-agents/{id}/trust", {
    params: { path: { id } },
    body,
  });
}

export async function updateAuthorizedAgent(id: string, agent: UpdateAuthorizedAgentRequest): Promise<AuthorizedAgent> {
  const { data: resData } = await client.PATCH("/authorized-agents/{id}", {
    params: { path: { id } },
    body: agent,
  });
  return resData;
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  await client.DELETE("/authorized-agents/{id}", {
    params: { path: { id } },
  });
}

export async function fetchDeviceInfo(id: string): Promise<DeviceInfo> {
  const { data: resData } = await client.GET("/devices/{id}/info", {
    params: { path: { id } },
  });
  // Cast required: generated schema returns {} but backend returns DeviceInfo — schema gap
  return resData as DeviceInfo;
}

