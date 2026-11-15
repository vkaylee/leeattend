import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

import { loadRuntimeFeatureFlags, isFeatureEnabled } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";

export const Route = createFileRoute("/_authenticated/_organization/ess")({
  beforeLoad: async () => {
    await loadRuntimeFeatureFlags();
    if (!isFeatureEnabled("ff_ess")) {
      throw redirect({ to: ROUTES.HOME });
    }
  },
  component: Outlet,
});
