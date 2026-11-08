import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EContractsSettingsPage from "@/pages/settings/econtracts-settings-page";


function EContractsSettingsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <EContractsSettingsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/settings/econtracts"
)({
  component: EContractsSettingsPageWithErrorBoundary,
});
