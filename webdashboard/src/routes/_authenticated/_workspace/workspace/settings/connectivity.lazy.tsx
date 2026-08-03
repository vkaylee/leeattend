import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ConnectivitySettings from "@/pages/settings/connectivity-settings";


function ConnectivitySettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ConnectivitySettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/connectivity')({
  component: ConnectivitySettingsWithErrorBoundary,
});
