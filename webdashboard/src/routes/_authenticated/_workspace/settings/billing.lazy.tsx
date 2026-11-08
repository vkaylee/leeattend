import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import WorkspaceBillingPage from "@/pages/workspace/workspace-billing-page";


function WorkspaceBillingPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <WorkspaceBillingPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/settings/billing')({
  component: WorkspaceBillingPageWithErrorBoundary,
});
