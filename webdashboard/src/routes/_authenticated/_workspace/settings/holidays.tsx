import { createFileRoute, redirect } from "@tanstack/react-router";

import { ROUTES } from "@/lib/routes";

export const Route = createFileRoute(
  "/_authenticated/_workspace/settings/holidays",
)({
  beforeLoad: () => {
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw redirect({ to: ROUTES.HOLIDAYS });
  },
});
