import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkspaceRolesPage from "@/pages/workspace/workspace-roles-page";


function WorkspaceRolesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkspaceRolesPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/settings/roles')({
  component: WorkspaceRolesPageWithErrorBoundary,
});
