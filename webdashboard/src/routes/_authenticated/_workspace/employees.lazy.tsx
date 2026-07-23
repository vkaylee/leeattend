import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EmployeesPage from "@/pages/employees-page";


function EmployeesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <EmployeesPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_workspace/employees')({
  component: EmployeesPageWithErrorBoundary,
});
