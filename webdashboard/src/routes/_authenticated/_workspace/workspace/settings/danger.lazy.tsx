import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkspaceDangerPage from "@/pages/workspace/workspace-danger-page";


function WorkspaceDangerPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkspaceDangerPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/danger')({
  component: WorkspaceDangerPageWithErrorBoundary,
});
