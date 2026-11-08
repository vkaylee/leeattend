import { createFileRoute, redirect } from "@tanstack/react-router";

import { isFeatureEnabled } from "@/lib/feature-flags";

export const Route = createFileRoute(
  "/_authenticated/_workspace/settings/econtracts"
)({
  beforeLoad: () => {
    if (!isFeatureEnabled("ff_econtracts")) {
      // eslint-disable-next-line @typescript-eslint/only-throw-error
      throw redirect({ to: "/" });
    }
  },
});
