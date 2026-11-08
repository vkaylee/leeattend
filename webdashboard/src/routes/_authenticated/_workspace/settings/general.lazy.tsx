import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import GeneralSettings from "@/pages/settings/general-settings";


function GeneralSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <GeneralSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/settings/general')({
  component: GeneralSettingsWithErrorBoundary,
});
