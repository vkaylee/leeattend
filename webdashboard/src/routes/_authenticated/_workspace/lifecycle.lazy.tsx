import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import PersonnelLifecyclePage from "@/pages/personnel/lifecycle-page";

function PersonnelLifecyclePageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <PersonnelLifecyclePage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_workspace/lifecycle")({
  component: PersonnelLifecyclePageWithErrorBoundary,
});
