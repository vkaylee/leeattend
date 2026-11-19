import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EmployeeCreatePage from "@/pages/employees/employee-create-page";

function EmployeeCreatePageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <EmployeeCreatePage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/employees_/new")({
  component: EmployeeCreatePageWithErrorBoundary,
});
