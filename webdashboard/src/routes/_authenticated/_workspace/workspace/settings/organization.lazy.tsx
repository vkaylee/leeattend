import { createLazyFileRoute } from "@tanstack/react-router";

import OrganizationSettings from "@/pages/settings/organization-settings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/organization')({
  component: OrganizationSettings,
});
