import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ConditionTypesPage from "@/pages/settings/condition-types-page";


function ConditionTypesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ConditionTypesPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/condition-types')({
  component: ConditionTypesPageWithErrorBoundary,
});
