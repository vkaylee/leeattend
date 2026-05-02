import { apiFetch } from "./helpers";

export type ClearDbGroup = "attendance" | "payroll" | "rosters" | "employees" | "full_reset" | "seed";

export const debugApi = {
  getUsers: async () => {
    return apiFetch("/debug/users");
  },

  clearDb: async (workspaceId: string, group: ClearDbGroup) => {
    return apiFetch(`/debug/workspaces/${workspaceId ?? ""}/clear`, {
      method: "POST",
      body: JSON.stringify({ group }),
    });
  },

  seedData: async (workspaceId: string, _scenario: string) => {
    // We use the clear endpoint with group='seed' for now as it's a debug command
    return apiFetch(`/debug/workspaces/${workspaceId ?? ""}/clear`, {
      method: "POST",
      body: JSON.stringify({ group: "seed" }),
    });
  }
};
