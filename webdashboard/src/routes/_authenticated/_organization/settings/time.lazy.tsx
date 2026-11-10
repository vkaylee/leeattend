import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import TimeSettings from "@/pages/settings/time-settings";


function TimeSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <TimeSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/time')({
  component: TimeSettingsWithErrorBoundary,
});
