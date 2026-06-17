 
import { createLazyFileRoute } from "@tanstack/react-router";

import { ESSDocumentsPage } from "@/pages/ess/ess-documents-page";

export const Route = createLazyFileRoute("/_authenticated/_workspace/ess/documents")({
  component: ESSDocumentsPage,
});
