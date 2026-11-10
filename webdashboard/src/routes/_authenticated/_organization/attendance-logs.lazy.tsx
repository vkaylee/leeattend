import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import AttendanceLogsPage from "@/pages/attendance-logs-page";


function AttendanceLogsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <AttendanceLogsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/attendance-logs')({
  component: AttendanceLogsPageWithErrorBoundary,
});
