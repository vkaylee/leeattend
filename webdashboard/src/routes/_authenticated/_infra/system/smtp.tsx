import { createFileRoute, redirect } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createFileRoute("/_authenticated/_infra/system/smtp")({
  beforeLoad: () => {
    throw redirect({ to: ROUTES.SETTINGS.SMTP });
  },
});
