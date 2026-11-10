import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import OvertimeSettings from "@/pages/settings/overtime-settings";


function OvertimeSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <OvertimeSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/overtime')({
  component: OvertimeSettingsWithErrorBoundary,
});
