import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ContractsPage from "@/pages/contracts-page";

function ContractsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ContractsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/contracts")({
  component: ContractsPageWithErrorBoundary,
});
