import { createLazyFileRoute } from "@tanstack/react-router";

import SystemRolesPage from "@/pages/system-roles-page";

export const Route = createLazyFileRoute('/_authenticated/_infra/system/roles')({
  component: SystemRolesPage,
});
