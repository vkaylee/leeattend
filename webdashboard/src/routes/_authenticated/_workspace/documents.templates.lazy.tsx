/* eslint-disable */
import { createLazyFileRoute } from "@tanstack/react-router";

import { DocumentTemplatesPage } from "@/pages/documents/document-templates-page";

export const Route = createLazyFileRoute("/_authenticated/_workspace/documents/templates")({
  component: DocumentTemplatesPage,
});
