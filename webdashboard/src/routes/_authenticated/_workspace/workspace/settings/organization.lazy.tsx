import { createLazyFileRoute } from "@tanstack/react-router";
import OrganizationSettings from "@/pages/settings/OrganizationSettings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/organization')({
  component: OrganizationSettings,
});
