import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import LeaveSettings from "@/pages/settings/leave-settings";


function LeaveSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <LeaveSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/leave')({
  component: LeaveSettingsWithErrorBoundary,
});
