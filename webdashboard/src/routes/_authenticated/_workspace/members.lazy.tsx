import { createLazyFileRoute, Navigate } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createLazyFileRoute("/_authenticated/_workspace/members")({
  component: () => <Navigate to={ROUTES.SETTINGS.MEMBERS} replace />,
});
