import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import OrganizationBillingPage from "@/pages/organization/organization-billing-page";


function OrganizationBillingPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <OrganizationBillingPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/settings/billing')({
  component: OrganizationBillingPageWithErrorBoundary,
});
