import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import OrganizationPermissionsPage from "@/pages/organization/organization-permissions-page";


function OrganizationPermissionsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <OrganizationPermissionsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/settings/permissions')({
  component: OrganizationPermissionsPageWithErrorBoundary,
});
