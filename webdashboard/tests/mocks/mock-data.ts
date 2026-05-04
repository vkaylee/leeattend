import { Device } from '../../src/lib/api/types';

export const MOCK_WORKSPACE = {
  id: "ws-mock-123",
  name: "Mock Enterprise",
  timezone: "Asia/Ho_Chi_Minh",
  rounding_policy: "nearest",
  work_unit_minutes: 30,
  early_arrival_strategy: "block" as const,
  created_at: "2026-01-01T00:00:00Z",
  date_format: "DD/MM/YYYY",
  default_language: "vi",
  theme_primary_color: "#7c3aed",
  time_format: "24h",
};

export const MOCK_USER = {
  id: "user-mock-1",
  email: "test@example.com",
  full_name: "Mock Admin",
  avatar_url: null,
  preferred_language: "vi"
};

export const MOCK_EMPLOYEES = [
  { id: "emp-mock-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001", department_id: "dept-1", status: "active" },
  { id: "emp-mock-2", first_name: "Jane", last_name: "Smith", employee_id_number: "EMP002", department_id: "dept-1", status: "active" }
];

export const MOCK_DEPARTMENTS = [
  { id: "dept-1", workspace_id: "ws-mock-123", name: "Kỹ thuật", code: "TECH" }
];

export const MOCK_AGENTS = [
  {
    id: "agent-mock-1",
    name: "Main Agent",
    token: "tok-abc123",
    is_trusted: true,
    last_ip: "10.0.0.1",
    last_seen_at: new Date().toISOString(),
    created_at: "2025-01-01T00:00:00Z",
    is_active: true,
    max_concurrent_workers: 5,
  }
];

export const MOCK_DEVICES_INITIAL: Device[] = [
  {
    id: "device-mock-1",
    agent_id: "agent-mock-1",
    name: "MAIN ENTRANCE",
    ip: "192.168.1.100",
    port: 4370,
    is_enabled: true,
    brand: "zkteco",
    password: "",
    timezone: "UTC",
    last_status: "online",
    created_at: "2025-01-01T00:00:00Z",
    workspace_id: "ws-mock-123",
  },
  {
    id: "device-mock-2",
    agent_id: "agent-mock-1",
    name: "BACK DOOR",
    ip: "192.168.1.101",
    port: 4370,
    is_enabled: false,
    brand: "mock",
    password: "1234",
    timezone: "Asia/Ho_Chi_Minh",
    last_status: "offline",
    created_at: "2025-01-02T00:00:00Z",
    workspace_id: "ws-mock-123",
  },
];
