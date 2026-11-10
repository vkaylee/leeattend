import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import OrgChartPage from "@/pages/org-chart-page";


function OrgChartPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <OrgChartPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute('/_authenticated/_organization/org-chart')({
  component: OrgChartPageWithErrorBoundary,
});
