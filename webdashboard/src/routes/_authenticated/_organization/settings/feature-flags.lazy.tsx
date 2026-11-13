import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import FeatureFlagsPage from "@/pages/settings/feature-flags-page";

function FeatureFlagsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <FeatureFlagsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/settings/feature-flags")({
  component: FeatureFlagsPageWithErrorBoundary,
});
