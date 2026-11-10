import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { StorageSettings } from "@/pages/settings/storage-settings";


function StorageSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <StorageSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute(
  "/_authenticated/_organization/settings/storage"
)({
  component: StorageSettingsWithErrorBoundary,
});
