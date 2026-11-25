import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchMyPermissions } from "@/lib/api";
import { adjustmentSearchSchema } from "@/pages/attendance/search-schema";

const ADJUSTMENTS_ROUTE_PERMISSIONS = [
  "attendance:adjustments:approve",
  "attendance:adjustments:manage_behalf",
  "attendance:adjustments:submit_on_behalf",
  "attendance:adjustments:submit",
] as const;

export const Route = createFileRoute("/_authenticated/_organization/attendance-adjustments")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!ADJUSTMENTS_ROUTE_PERMISSIONS.some((p) => permissions.includes(p))) {
        throw redirect({ to: "/" });
      }
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => adjustmentSearchSchema.parse(search),
});
