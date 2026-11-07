import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import SmtpSettingsPage from "@/pages/system-admin/smtp-settings-page";

function SmtpSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <SmtpSettingsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/workspace/settings/smtp",
)({
  component: SmtpSettingsWithErrorBoundary,
});
