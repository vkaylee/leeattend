 
import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { ESSDocumentsPage } from "@/pages/ess/ess-documents-page";


function ESSDocumentsPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ESSDocumentsPage />
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute("/_authenticated/_organization/ess/documents")({
  component: ESSDocumentsPageWithErrorBoundary,
});
