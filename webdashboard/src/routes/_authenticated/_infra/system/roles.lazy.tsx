import { createLazyFileRoute } from "@tanstack/react-router";

import SystemRolesPage from "@/pages/SystemRolesPage";

export const Route = createLazyFileRoute('/_authenticated/_infra/system/roles')({
  component: SystemRolesPage,
});
