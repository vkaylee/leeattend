import { createLazyFileRoute } from "@tanstack/react-router";

import TenantsPage from "@/pages/system-admin/tenants-page";

export const Route = createLazyFileRoute('/_authenticated/_infra/system/tenants')({
  component: TenantsPage,
});
