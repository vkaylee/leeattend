import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchMyPermissions } from "@/lib/api";

export const Route = createFileRoute(
  "/_authenticated/_organization/settings/smtp",
)({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("organization:settings:read") && !permissions.includes("infra:monitor")) {
        throw redirect({ to: "/" });
      }
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
});
