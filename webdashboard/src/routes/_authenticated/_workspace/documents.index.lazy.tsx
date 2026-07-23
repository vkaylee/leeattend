 
import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { DocumentManagementPage } from "@/pages/documents/document-management-page";


function DocumentManagementPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <DocumentManagementPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute("/_authenticated/_workspace/documents/")({
  component: DocumentManagementPageWithErrorBoundary,
});
