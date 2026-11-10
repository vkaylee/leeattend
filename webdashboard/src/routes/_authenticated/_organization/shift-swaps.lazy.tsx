import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ShiftSwapsPage from "@/pages/shift-swaps-page";


function ShiftSwapsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ShiftSwapsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/shift-swaps')({
  component: ShiftSwapsPageWithErrorBoundary,
});
