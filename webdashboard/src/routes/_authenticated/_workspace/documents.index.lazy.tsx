/* eslint-disable */
import { createLazyFileRoute } from "@tanstack/react-router";

import { DocumentManagementPage } from "@/pages/documents/document-management-page";

export const Route = createLazyFileRoute("/_authenticated/_workspace/documents/")({
  component: DocumentManagementPage,
});
