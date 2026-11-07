import { createLazyFileRoute, Navigate } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/workspace/people",
)({
  component: () => <Navigate to={ROUTES.SETTINGS.MEMBERS} replace />,
});
