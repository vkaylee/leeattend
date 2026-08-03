import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ShiftsPage from "@/pages/shifts-page";


function ShiftsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ShiftsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/shifts/')({
  component: ShiftsPageWithErrorBoundary,
});
