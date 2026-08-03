import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import PayrollSettings from "@/pages/settings/payroll-settings";


function PayrollSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <PayrollSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/payroll')({
  component: PayrollSettingsWithErrorBoundary,
});
