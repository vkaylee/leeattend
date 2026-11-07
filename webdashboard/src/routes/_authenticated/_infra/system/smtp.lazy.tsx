import { createLazyFileRoute, Navigate } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createLazyFileRoute("/_authenticated/_infra/system/smtp")({
  component: () => <Navigate to={ROUTES.SETTINGS.SMTP} replace />,
});
