import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import DashboardPage from "@/pages/dashboard-page";


function DashboardPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <DashboardPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute("/_authenticated/_workspace/dashboard")({
  component: DashboardPageWithErrorBoundary,
});
