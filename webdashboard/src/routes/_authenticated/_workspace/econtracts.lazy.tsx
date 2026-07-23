import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EContractsPage from "@/pages/econtracts-page";


function EContractsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <EContractsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute("/_authenticated/_workspace/econtracts")({
  component: EContractsPageWithErrorBoundary,
});
