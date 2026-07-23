import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import DepartmentsPage from "@/pages/departments-page";


function DepartmentsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <DepartmentsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_workspace/departments')({
  component: DepartmentsPageWithErrorBoundary,
});
