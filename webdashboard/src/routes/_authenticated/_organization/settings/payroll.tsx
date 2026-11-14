import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchMyPermissions } from "@/lib/api";
import { payrollSettingsSearchSchema } from "@/pages/settings/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/settings/payroll")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("payroll:read")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => payrollSettingsSearchSchema.parse(search),
});
