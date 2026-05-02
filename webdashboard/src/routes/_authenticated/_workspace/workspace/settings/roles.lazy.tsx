import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspaceRolesPage from "@/pages/workspace/workspace-roles-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/roles')({
  component: WorkspaceRolesPage,
});
