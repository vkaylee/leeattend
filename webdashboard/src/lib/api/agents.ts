import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { AgentStatus, Device, AuthorizedAgent, DeviceInfo } from "./types";

export async function fetchAgents(): Promise<AgentStatus[]> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/agents`);
  if (!response.ok) throw new Error("Failed to fetch agents");
  return response.json() as Promise<AgentStatus[]>;
}

 
 
export async function sendAgentCommand(agentId: string, command: unknown): Promise<unknown> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/agent/${agentId ?? ""}/command`, {
    method: "POST",
    body: JSON.stringify(command),
  });
  return response.json() as Promise<unknown>;
}

export async function fetchDevices(): Promise<Device[]> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/devices`);
  if (!response.ok) throw new Error("Failed to fetch devices");
  return response.json() as Promise<Device[]>;
}

export async function createDevice(
  device: Partial<Device>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/devices`, {
    method: "POST",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to create device");
  return response.json() as Promise<{ id: string }>;
}

export async function updateDevice(
  id: string,
  device: Partial<Device>,
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/devices/${id ?? ""}`, {
    method: "PUT",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to update device");
}

export async function deleteDevice(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/devices/${id ?? ""}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete device");
}

export async function fetchAuthorizedAgents(
  includeInactive = false,
): Promise<AuthorizedAgent[]> {
  const url = includeInactive
    ? `${API_BASE_URL ?? ""}/authorized-agents?include_inactive=true`
    : `${API_BASE_URL ?? ""}/authorized-agents`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch authorized agents");
  return response.json() as Promise<AuthorizedAgent[]>;
}

export async function fetchAuthorizedAgentById(id: string): Promise<AuthorizedAgent> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/authorized-agents/${id ?? ""}`);
  if (!response.ok) throw new Error("Failed to fetch authorized agent");
  return response.json() as Promise<AuthorizedAgent>;
}

export async function createAuthorizedAgent(
  agent: Partial<AuthorizedAgent>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/authorized-agents`, {
    method: "POST",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to create authorized agent");
  return response.json() as Promise<{ id: string }>;
}

export async function updateAgentTrust(
  id: string,
  is_trusted: boolean,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL ?? ""}/authorized-agents/${id ?? ""}/trust`,
    {
      method: "PUT",
      body: JSON.stringify({ is_trusted }),
    },
  );
  if (!response.ok) throw new Error("Failed to update agent trust");
}

export async function updateAuthorizedAgent(
  id: string,
  agent: Partial<AuthorizedAgent>,
): Promise<AuthorizedAgent> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/authorized-agents/${id ?? ""}`, {
    method: "PATCH",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to update authorized agent");
  return response.json() as Promise<AuthorizedAgent>;
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/authorized-agents/${id ?? ""}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete authorized agent");
}

export async function fetchDeviceInfo(id: string): Promise<DeviceInfo> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/devices/${id ?? ""}/info`);
  if (!response.ok) throw new Error("Failed to fetch device info");
  return response.json() as Promise<DeviceInfo>;
}
