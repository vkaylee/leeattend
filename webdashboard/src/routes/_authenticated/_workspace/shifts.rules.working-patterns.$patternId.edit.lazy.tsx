import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkingPatternsEditorPage from "@/pages/working-patterns-editor-page";


function WorkingPatternsEditorPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkingPatternsEditorPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/shifts/rules/working-patterns/$patternId/edit')({
  component: WorkingPatternsEditorPageWithErrorBoundary,
});
