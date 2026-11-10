 
import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { DocumentTemplatesPage } from "@/pages/documents/document-templates-page";


function DocumentTemplatesPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <DocumentTemplatesPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute("/_authenticated/_organization/documents/templates")({
  component: DocumentTemplatesPageWithErrorBoundary,
});
