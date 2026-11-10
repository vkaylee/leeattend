import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkflowEditorPage from "@/pages/settings/workflow-editor-page";


function WorkflowEditorPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkflowEditorPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute(
  '/_authenticated/_organization/settings/workflows/$workflowId/edit',
)({
  component: WorkflowEditorPageWithErrorBoundary,
});
