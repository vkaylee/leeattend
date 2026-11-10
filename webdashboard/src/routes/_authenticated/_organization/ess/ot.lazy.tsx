import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EssOtPage from "@/pages/ess/ot-page";


function EssOtPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <EssOtPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/ess/ot')({
  component: EssOtPageWithErrorBoundary,
});
