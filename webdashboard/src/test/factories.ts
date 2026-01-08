import { User, Workspace, AuthorizedAgent, WorkspaceMember } from "@/lib/api";

export const userFactory = (overrides: Partial<User> = {}): User => {
  const id = crypto.randomUUID();
  return {
    id,
    email: `user_${id.slice(0, 8)}@example.com`,
    created_at: new Date().toISOString(),
    full_name: `User ${id.slice(0, 4)}`,
    avatar_url: null,
    bio: null,
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
    ...overrides,
  };
};
