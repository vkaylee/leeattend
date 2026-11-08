import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import OrganizationSettings from "@/pages/settings/organization-settings";


function OrganizationSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <OrganizationSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/settings/organization')({
  component: OrganizationSettingsWithErrorBoundary,
});
