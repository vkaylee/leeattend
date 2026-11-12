import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import EContractsPlaceholderPage from "@/pages/econtracts-page";


function EContractsPlaceholderWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <EContractsPlaceholderPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute("/_authenticated/_organization/econtracts")({
  component: EContractsPlaceholderWithErrorBoundary,
});
