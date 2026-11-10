import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import LifecycleTemplatesPage from "@/pages/settings/lifecycle-templates-page";


function LifecycleTemplatesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <LifecycleTemplatesPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/lifecycle-templates')({
  component: LifecycleTemplatesPageWithErrorBoundary,
});
