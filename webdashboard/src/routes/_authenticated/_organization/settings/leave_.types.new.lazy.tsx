import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { NewLeaveTypePage } from "@/pages/settings/new-leave-type-page";

function NewLeaveTypePageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <NewLeaveTypePage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute(
  "/_authenticated/_organization/settings/leave_/types/new",
)({
  component: NewLeaveTypePageWithErrorBoundary,
});
