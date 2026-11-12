import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EmploymentPhasesPage from "@/pages/personnel/employment-phases-page";

function EmploymentPhasesPageWithErrorBoundary() {
  return <ErrorBoundary variant="full"><EmploymentPhasesPage /></ErrorBoundary>;
}

export const Route = createLazyFileRoute("/_authenticated/_organization/employment-phases")({
  component: EmploymentPhasesPageWithErrorBoundary,
});
