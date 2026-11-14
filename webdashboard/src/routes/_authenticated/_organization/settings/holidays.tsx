import { createFileRoute, redirect } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createFileRoute(
  "/_authenticated/_organization/settings/holidays",
)({
  beforeLoad: () => {
    throw redirect({ to: ROUTES.HOLIDAYS });
  },
});
