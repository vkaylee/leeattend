import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ReportsPage from "@/pages/reports-page";


function ReportsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ReportsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/reports')({
  component: ReportsPageWithErrorBoundary,
});
