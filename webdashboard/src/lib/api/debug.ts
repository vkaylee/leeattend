import { apiFetch } from "./helpers";

export type ClearDbGroup = "attendance" | "payroll" | "rosters" | "full_reset";

export const debugApi = {
  getUsers: async () => {
    return apiFetch("/debug/users");
  },

  clearDb: async (workspaceId: string, group: ClearDbGroup) => {
    return apiFetch(`/debug/workspaces/${workspaceId}/clear`, {
      method: "POST",
      body: JSON.stringify({ group }),
    });
  },

  seedData: async (workspaceId: string, scenario: string) => {
    // Note: We might implement seed logic in the future
    return apiFetch(`/debug/workspaces/${workspaceId}/seed`, {
      method: "POST",
      body: JSON.stringify({ scenario }),
    });
  }
};
