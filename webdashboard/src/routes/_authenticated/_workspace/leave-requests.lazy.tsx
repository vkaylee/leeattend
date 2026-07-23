import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import LeaveRequestsPage from "@/pages/leave-requests-page";


function LeaveRequestsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <LeaveRequestsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_workspace/leave-requests')({
  component: LeaveRequestsPageWithErrorBoundary,
});
