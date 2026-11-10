import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import PersonnelWorkingConditionsPage from "@/pages/personnel/working-conditions-page";

function PersonnelWorkingConditionsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <PersonnelWorkingConditionsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/working-conditions")({
  component: PersonnelWorkingConditionsPageWithErrorBoundary,
});
