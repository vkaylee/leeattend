import { createLazyFileRoute } from "@tanstack/react-router";

import SystemTenantsPage from "@/pages/system-tenants-page";

export const Route = createLazyFileRoute("/_authenticated/_infra/system/tenants")({
  component: SystemTenantsPage,
});
