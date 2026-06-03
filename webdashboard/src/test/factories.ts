
import { User, Workspace, AuthorizedAgent, WorkspaceMember, Device } from "@/lib/api";

export const userFactory = (overrides: Partial<User> = {}): User => {
  const id = crypto.randomUUID();
  return {
    id,
    email: `user_${id.slice(0, 8)}@example.com`,
    full_name: `User ${id.slice(0, 4)}`,
    avatar_url: undefined,
    bio: undefined,
    created_at: new Date().toISOString(),
    ...overrides,
  };
};

export const workspaceFactory = (
  overrides: Partial<Workspace> = {},
): Workspace => {
  const id = crypto.randomUUID();
  return {
    id,
    name: `Workspace ${id.slice(0, 8)}`,
    date_format: "DD/MM/YYYY",
    time_format: "24h",
    timezone: "Asia/Ho_Chi_Minh",
    theme_primary_color: "#10b981",
    default_language: "en",
    default_holiday_ot_multiplier: "1.5",
    holiday_overnight_strategy: "split",
    created_at: new Date().toISOString(),
    ...overrides,
  };
};

export const agentFactory = (
  overrides: Partial<AuthorizedAgent> = {},
): AuthorizedAgent => {
  return {
    id: `AGENT_${Math.floor(Math.random() * 1000)}`,
    name: "Test Agent",
    token: "secret_token",
    is_trusted: true,
    is_active: true,
    max_concurrent_workers: 2,
    success_cooldown_secs: 300,
    retention_days: 365,
    last_ip: "127.0.0.1",
    last_seen_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
    workspace_id: crypto.randomUUID(),
    ...overrides,
  };
};

export const deviceFactory = (
  overrides: Partial<Device> = {},
): Device => {
  return {
    id: `DEV_${Math.floor(Math.random() * 1000)}`,
    agent_id: "AGENT_1",
    name: "Test Device",
    ip: "192.168.1.100",
    port: 4370,
    brand: "zkteco",
    is_enabled: true,
    timezone: "UTC",
    last_status: "online",
    max_concurrent_workers: 1,
    created_at: new Date().toISOString(),
    workspace_id: crypto.randomUUID(),
    ...overrides,
  };
};

export const memberFactory = (
  overrides: Partial<WorkspaceMember> = {},
): WorkspaceMember => {
  return {
    workspace_id: crypto.randomUUID(),
    user_id: crypto.randomUUID(),
    role_id: "role-member",
    extra_role_ids: [],
    email: "test@example.com",
    ...overrides,
  };
};
