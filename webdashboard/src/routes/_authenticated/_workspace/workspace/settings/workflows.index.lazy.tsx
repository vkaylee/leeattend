import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkflowsSettings from "@/pages/settings/workflows-settings";


function WorkflowsSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkflowsSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_workspace/workspace/settings/workflows/")({
  component: WorkflowsSettingsWithErrorBoundary,
});
