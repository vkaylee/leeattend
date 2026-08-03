import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkspacePermissionsPage from "@/pages/workspace/workspace-permissions-page";


function WorkspacePermissionsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkspacePermissionsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/permissions')({
  component: WorkspacePermissionsPageWithErrorBoundary,
});
