import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchMyPermissions } from "@/lib/api";

export const Route = createFileRoute('/_authenticated/_organization/reports')({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("reports:read")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
});
