import { apiFetch } from "./helpers";

export type ClearDbGroup = "attendance" | "payroll" | "rosters" | "employees" | "full_reset" | "seed";

export const debugApi = {
  getUsers: async () => {
    return apiFetch("/v1/debug/users");
  },

  clearDb: async (workspaceId: string, group: ClearDbGroup) => {
    return apiFetch(`/v1/debug/workspaces/${workspaceId}/clear`, {
      method: "POST",
      body: JSON.stringify({ group }),
    });
  },
};
