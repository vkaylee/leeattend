import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ShiftsRulesPage from "@/pages/shifts-rules-page";


function ShiftsRulesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ShiftsRulesPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_workspace/shifts/rules/")({
  component: ShiftsRulesPageWithErrorBoundary,
});
