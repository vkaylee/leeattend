import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { AgentStatus, Device, AuthorizedAgent } from "./types";

export async function fetchAgents(): Promise<AgentStatus[]> {
  const response = await apiFetch(`${API_BASE_URL}/agents`);
  if (!response.ok) throw new Error("Failed to fetch agents");
  return response.json();
}

export async function sendAgentCommand(agentId: string, command: any): Promise<any> {
  const response = await apiFetch(`${API_BASE_URL}/agent/${agentId}/command`, {
    method: "POST",
    body: JSON.stringify(command),
  });
  return response.json();
}

export async function fetchDevices(): Promise<Device[]> {
  const response = await apiFetch(`${API_BASE_URL}/devices`);
  if (!response.ok) throw new Error("Failed to fetch devices");
  return response.json();
}

export async function createDevice(
  device: Partial<Device>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL}/devices`, {
    method: "POST",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to create device");
  return response.json();
}

export async function updateDevice(
  id: string,
  device: Partial<Device>,
): Promise<Device> {
  const response = await apiFetch(`${API_BASE_URL}/devices/${id}`, {
    method: "PUT",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to update device");
  return response.json();
}

export async function deleteDevice(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/devices/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete device");
}

export async function fetchAuthorizedAgents(
  includeInactive: boolean = false,
): Promise<AuthorizedAgent[]> {
  const url = includeInactive
    ? `${API_BASE_URL}/authorized-agents?include_inactive=true`
    : `${API_BASE_URL}/authorized-agents`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch authorized agents");
  return response.json();
}

export async function fetchAuthorizedAgentById(id: string): Promise<AuthorizedAgent> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`);
  if (!response.ok) throw new Error("Failed to fetch authorized agent");
  return response.json();
}

export async function createAuthorizedAgent(
  agent: Partial<AuthorizedAgent>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents`, {
    method: "POST",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to create authorized agent");
  return response.json();
}

export async function updateAgentTrust(
  id: string,
  is_trusted: boolean,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/authorized-agents/${id}/trust`,
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
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: "PATCH",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to update authorized agent");
  return response.json();
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete authorized agent");
}
