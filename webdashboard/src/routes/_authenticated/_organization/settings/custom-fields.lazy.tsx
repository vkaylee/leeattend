import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import CustomFieldsPage from "@/pages/settings/custom-fields-page";


function CustomFieldsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <CustomFieldsPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/custom-fields')({
  component: CustomFieldsPageWithErrorBoundary,
});
