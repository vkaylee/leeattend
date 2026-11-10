import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import OtApprovalsPage from "@/pages/approvals/ot-approvals-page";


function OtApprovalsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <OtApprovalsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/approvals/ot')({
  component: OtApprovalsPageWithErrorBoundary,
});
