import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import OrganizationRolesPage from "@/pages/organization/organization-roles-page";


function OrganizationRolesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <OrganizationRolesPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/roles')({
  component: OrganizationRolesPageWithErrorBoundary,
});
