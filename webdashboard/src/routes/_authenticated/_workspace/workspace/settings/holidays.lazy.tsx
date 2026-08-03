import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import HolidaysPage from "@/pages/settings/holidays-page";


function HolidaysPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <HolidaysPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/holidays')({
  component: HolidaysPageWithErrorBoundary,
});
