import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import AssignmentRulesPage from "@/pages/assignment-rules-page";


function AssignmentRulesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <AssignmentRulesPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/assignment-rules/')({
  component: AssignmentRulesPageWithErrorBoundary,
});
