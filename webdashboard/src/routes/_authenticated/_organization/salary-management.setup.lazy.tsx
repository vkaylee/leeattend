import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import SalarySetupPage from "@/pages/salary-management/salary-setup-page";

function SalarySetupPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <SalarySetupPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/salary-management/setup")({
  component: SalarySetupPageWithErrorBoundary,
});
