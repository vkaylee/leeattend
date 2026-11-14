import { createFileRoute, redirect } from "@tanstack/react-router";

import { isFeatureEnabled } from "@/lib/feature-flags";

export const Route = createFileRoute("/_authenticated/_organization/econtracts")({
  beforeLoad: () => {
    if (!isFeatureEnabled("ff_econtracts")) {
      throw redirect({ to: "/contracts" });
    }
  },
});
