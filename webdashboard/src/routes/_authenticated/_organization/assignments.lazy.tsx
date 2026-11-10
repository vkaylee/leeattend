import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import PersonnelAssignmentsPage from "@/pages/personnel/assignments-page";

function PersonnelAssignmentsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <PersonnelAssignmentsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/assignments")({
  component: PersonnelAssignmentsPageWithErrorBoundary,
});
