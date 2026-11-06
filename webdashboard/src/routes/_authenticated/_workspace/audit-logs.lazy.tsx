import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import AuditLogPage from "@/pages/audit-logs/audit-log-page";

function AuditLogPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <AuditLogPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_workspace/audit-logs")({
  component: AuditLogPageWithErrorBoundary,
});
