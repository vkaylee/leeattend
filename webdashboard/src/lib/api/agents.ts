import { client } from "./client";
import { AgentStatus, Device, AuthorizedAgent, DeviceInfo } from "./types";

export async function fetchAgents(): Promise<AgentStatus[]> {
  const { data: resData } = await client.GET("/v1/agents");
  if (!resData) throw new Error("No response data");
  return resData as unknown as AgentStatus[];
}

export async function sendAgentCommand(agentId: string, command: unknown): Promise<unknown> {
  const { data: resData } = await client.POST("/v1/agent/{agent_id}/command", {
    params: { path: { agent_id: agentId } },
    body: command as never,
  });
  return resData;
}

export async function fetchDevices(): Promise<Device[]> {
  const { data: resData } = await client.GET("/v1/devices");
  if (!resData) throw new Error("No response data");
  return resData as unknown as Device[];
}

export async function createDevice(device: Partial<Device>): Promise<{ id: string }> {
  const { data: resData } = await client.POST("/v1/devices", {
    body: device as never,
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function updateDevice(id: string, device: Partial<Device>): Promise<void> {
  await client.PUT("/v1/devices/{id}", {
    params: { path: { id } },
    body: device as never,
  });
}

export async function deleteDevice(id: string): Promise<void> {
  await client.DELETE("/v1/devices/{id}", {
    params: { path: { id } },
  });
}

export async function fetchAuthorizedAgents(includeInactive = false): Promise<AuthorizedAgent[]> {
  const { data: resData } = await client.GET("/v1/authorized-agents", {
    params: {
      query: { include_inactive: includeInactive },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as AuthorizedAgent[];
}

export async function fetchAuthorizedAgentById(id: string): Promise<AuthorizedAgent> {
  const { data: resData } = await client.GET("/v1/authorized-agents/{id}", {
    params: { path: { id } },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as AuthorizedAgent;
}

export async function createAuthorizedAgent(agent: Partial<AuthorizedAgent>): Promise<{ id: string }> {
  const { data: resData } = await client.POST("/v1/authorized-agents", {
    body: agent as never,
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function updateAgentTrust(id: string, is_trusted: boolean): Promise<void> {
  await client.PUT("/v1/authorized-agents/{id}/trust", {
    params: { path: { id } },
    body: { is_trusted } as never,
  });
}

export async function updateAuthorizedAgent(id: string, agent: Partial<AuthorizedAgent>): Promise<AuthorizedAgent> {
  const { data: resData } = await client.PATCH("/v1/authorized-agents/{id}", {
    params: { path: { id } },
    body: agent as never,
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as AuthorizedAgent;
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  await client.DELETE("/v1/authorized-agents/{id}", {
    params: { path: { id } },
  });
}

export async function fetchDeviceInfo(id: string): Promise<DeviceInfo> {
  const { data: resData } = await client.GET("/v1/devices/{id}/info", {
    params: { path: { id } },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as DeviceInfo;
}

