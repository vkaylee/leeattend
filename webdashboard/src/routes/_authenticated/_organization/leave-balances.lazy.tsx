import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import LeaveBalancesPage from "@/pages/leave-balances-page";


function LeaveBalancesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <LeaveBalancesPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/leave-balances')({
  component: LeaveBalancesPageWithErrorBoundary,
});
