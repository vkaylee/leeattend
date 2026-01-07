/**
 * Lấy địa chỉ API Base từ biến môi trường.
 * Mặc định là http://localhost:3000 nếu không được cấu hình.
 */
export const API_BASE_URL = (process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000').replace(/\/$/, '');

/**
 * Tự động tạo URL WebSocket dựa trên API_BASE_URL
 */
export const SOCKET_URL = API_BASE_URL.replace(/^http/, 'ws') + '/ws';

export interface DeviceStatus {
  device_id: string;
  status: 'online' | 'offline';
  last_sync?: string;
  error?: string;
}

export interface AgentStatus {
  agent_id: string;
  last_seen: string;
  is_online: boolean;
  devices: DeviceStatus[];
}

export interface Device {
  id: string;
  agent_id: string;
  name: string;
  ip: string;
  port: number;
  is_enabled: boolean;
  last_status: string;
  created_at: string;
}

export async function fetchAgents(): Promise<AgentStatus[]> {
  const response = await fetch(`${API_BASE_URL}/agents`);
  if (!response.ok) throw new Error('Failed to fetch agents');
  return response.json();
}

export async function fetchDevices(): Promise<Device[]> {
  const response = await fetch(`${API_BASE_URL}/devices`);
  if (!response.ok) throw new Error('Failed to fetch devices');
  return response.json();
}

export async function createDevice(device: Partial<Device>): Promise<{ id: string }> {
  const response = await fetch(`${API_BASE_URL}/devices`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error('Failed to create device');
  return response.json();
}

export async function deleteDevice(id: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/devices/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error('Failed to delete device');
}