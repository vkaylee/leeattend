import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkingPatternsPage from "@/pages/working-patterns-page";


function WorkingPatternsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkingPatternsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/working-patterns/')({
  component: WorkingPatternsPageWithErrorBoundary,
});
