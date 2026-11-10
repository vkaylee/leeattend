import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EmployeeDetailPage from "@/pages/employees/employee-detail-page";

function EmployeeDetailPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <EmployeeDetailPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute(
  "/_authenticated/_organization/employees_/$employeeId",
)({
  component: EmployeeDetailPageWithErrorBoundary,
});
