import { useMutation, useQuery } from "@tanstack/react-query";

import { apiFetch } from "@/lib/api/helpers";
import { createQueryKey } from "@/lib/query-keys";

import {
  sanitizeDashboardPreferences,
  type DashboardLayoutPreferences,
} from "./use-dashboard-layout-presets";

export class DashboardPreferencesError extends Error {
  readonly status?: number;

  constructor(message: string, status?: number) {
    super(message);
    this.name = "DashboardPreferencesError";
    this.status = status;
  }
}

export type DashboardPreferencesUpdate = Partial<Omit<DashboardLayoutPreferences, "id" | "user_id">>;

export function useDashboardPreferences() {
  return useQuery<DashboardLayoutPreferences>({
    queryKey: createQueryKey("dashboard-preferences"),
    queryFn: async () => {
      const response = await apiFetch("/api/v1/dashboard/preferences");
      if (!response.ok) {
        throw new DashboardPreferencesError("Dashboard preferences could not be loaded.", response.status);
      }
      try {
        return sanitizeDashboardPreferences(await response.json());
      } catch {
        throw new DashboardPreferencesError("Dashboard preferences returned an invalid response.", response.status);
      }
    },
  });
}

export function useDashboardMutation() {
  return useMutation({
    mutationFn: async (update: DashboardPreferencesUpdate) => {
      const response = await apiFetch("/api/v1/dashboard/preferences", {
        method: "PUT",
        body: JSON.stringify(update),
      });
      if (!response.ok) {
        throw new DashboardPreferencesError("Dashboard preferences could not be saved.", response.status);
      }
      try {
        return sanitizeDashboardPreferences(await response.json());
      } catch {
        throw new DashboardPreferencesError("Dashboard preferences returned an invalid response.", response.status);
      }
    },
  });
}
