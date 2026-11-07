import { createLazyFileRoute, Navigate } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/workspace/settings/holidays",
)({
  component: () => <Navigate to={ROUTES.HOLIDAYS} replace />,
});
