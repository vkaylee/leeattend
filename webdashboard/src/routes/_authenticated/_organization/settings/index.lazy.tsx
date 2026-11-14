import { createLazyFileRoute, Navigate } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createLazyFileRoute(
  "/_authenticated/_organization/settings/"
)({
  component: () => <Navigate replace to={ROUTES.SETTINGS.GENERAL} />,
});
