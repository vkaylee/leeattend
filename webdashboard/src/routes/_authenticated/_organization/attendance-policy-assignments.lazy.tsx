import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import AttendancePolicyAssignmentsPage from "@/pages/personnel/attendance-policy-assignments-page";

function AttendancePolicyAssignmentsPageWithErrorBoundary() {
  return <ErrorBoundary variant="full"><AttendancePolicyAssignmentsPage /></ErrorBoundary>;
}

export const Route = createLazyFileRoute("/_authenticated/_organization/attendance-policy-assignments")({
  component: AttendancePolicyAssignmentsPageWithErrorBoundary,
});
