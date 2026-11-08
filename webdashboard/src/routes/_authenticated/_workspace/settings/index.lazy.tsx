import { createLazyFileRoute, Navigate } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/settings/"
)({
  component: () => <Navigate to={ROUTES.SETTINGS.GENERAL} replace />,
});
