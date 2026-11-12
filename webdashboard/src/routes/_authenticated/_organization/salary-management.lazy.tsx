import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import SalaryManagementPage from "@/pages/salary-management-page";

function SalaryManagementPageWithErrorBoundary() {
  return <ErrorBoundary variant="full"><SalaryManagementPage /></ErrorBoundary>;
}

export const Route = createLazyFileRoute("/_authenticated/_organization/salary-management")({
  component: SalaryManagementPageWithErrorBoundary,
});
