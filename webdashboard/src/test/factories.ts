import { User, Workspace, AuthorizedAgent, WorkspaceMember } from "@/lib/api";

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
    work_unit_minutes: 30, // Default value
    rounding_policy: "none", // Default value
    date_format: "DD/MM/YYYY", // Default value
    time_format: "24h", // Default value
    timezone: "Asia/Ho_Chi_Minh", // Default value
    theme_primary_color: "#7c3aed",
    default_language: "en",
    pairing_algorithm: "sequential",
    early_arrival_strategy: "block",
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
    last_ip: "127.0.0.1",
    last_seen_at: new Date().toISOString(),
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
    role: "member",
    email: "test@example.com",
    ...overrides,
  };
};
