import { createFileRoute, redirect } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/"
)({
  beforeLoad: () => {
    throw redirect({ to: ROUTES.SETTINGS.GENERAL });
  },
});
