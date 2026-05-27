import { apiClient } from "./client";
import { AgentStatus, Device, AuthorizedAgent, DeviceInfo } from "./types";

export async function fetchAgents(): Promise<AgentStatus[]> {
  const { data } = await apiClient.get<AgentStatus[]>("/agents");
  return data;
}

export async function sendAgentCommand(agentId: string, command: unknown): Promise<unknown> {
  const { data } = await apiClient.post<unknown>(`/agent/${agentId}/command`, command);
  return data;
}

export async function fetchDevices(): Promise<Device[]> {
  const { data } = await apiClient.get<Device[]>("/devices");
  return data;
}

export async function createDevice(device: Partial<Device>): Promise<{ id: string }> {
  const { data } = await apiClient.post<{ id: string }>("/devices", device);
  return data;
}

export async function updateDevice(id: string, device: Partial<Device>): Promise<void> {
  await apiClient.put(`/devices/${id}`, device);
}

export async function deleteDevice(id: string): Promise<void> {
  await apiClient.delete(`/devices/${id}`);
}

export async function fetchAuthorizedAgents(includeInactive = false): Promise<AuthorizedAgent[]> {
  const url = includeInactive ? "/authorized-agents?include_inactive=true" : "/authorized-agents";
  const { data } = await apiClient.get<AuthorizedAgent[]>(url);
  return data;
}

export async function fetchAuthorizedAgentById(id: string): Promise<AuthorizedAgent> {
  const { data } = await apiClient.get<AuthorizedAgent>(`/authorized-agents/${id}`);
  return data;
}

export async function createAuthorizedAgent(agent: Partial<AuthorizedAgent>): Promise<{ id: string }> {
  const { data } = await apiClient.post<{ id: string }>("/authorized-agents", agent);
  return data;
}

export async function updateAgentTrust(id: string, is_trusted: boolean): Promise<void> {
  await apiClient.put(`/authorized-agents/${id}/trust`, { is_trusted });
}

export async function updateAuthorizedAgent(id: string, agent: Partial<AuthorizedAgent>): Promise<AuthorizedAgent> {
  const { data } = await apiClient.patch<AuthorizedAgent>(`/authorized-agents/${id}`, agent);
  return data;
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  await apiClient.delete(`/authorized-agents/${id}`);
}

export async function fetchDeviceInfo(id: string): Promise<DeviceInfo> {
  const { data } = await apiClient.get<DeviceInfo>(`/devices/${id}/info`);
  return data;
}
