import { createFileRoute, redirect } from "@tanstack/react-router";

import AppShell from "@/components/layout/app-shell";
import { fetchMyPermissions } from "@/lib/api";
import { ROUTES } from "@/lib/routes";

function InfraAppShell() {
  return <AppShell variant="infra" />;
}

export const Route = createFileRoute("/_authenticated/_infra")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("infra:monitor")) {
        throw redirect({ to: ROUTES.HOME });
      }
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: ROUTES.HOME });
    }
  },
  component: InfraAppShell,
});
