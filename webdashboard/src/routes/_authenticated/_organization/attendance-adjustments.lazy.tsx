import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import AttendanceAdjustmentsPage from "@/pages/attendance-adjustments-page";


function AttendanceAdjustmentsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <AttendanceAdjustmentsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/attendance-adjustments')({
  component: AttendanceAdjustmentsPageWithErrorBoundary,
});
