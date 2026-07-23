import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import MyAttendancePage from "@/pages/ess/my-attendance-page";


function MyAttendancePageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <MyAttendancePage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/attendance')({
  component: MyAttendancePageWithErrorBoundary,
});
