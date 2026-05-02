import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspacePermissionsPage from "@/pages/workspace/workspace-permissions-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/permissions')({
  component: WorkspacePermissionsPage,
});
