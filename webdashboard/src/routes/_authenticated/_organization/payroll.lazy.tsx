import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import PayrollPage from "@/pages/payroll-page";


function PayrollPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <PayrollPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/payroll')({
  component: PayrollPageWithErrorBoundary,
});
