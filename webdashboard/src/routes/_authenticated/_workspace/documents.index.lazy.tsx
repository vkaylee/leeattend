import { createLazyFileRoute } from "@tanstack/react-router";
import { DocumentManagementPage } from "@/pages/documents/DocumentManagementPage";

export const Route = createLazyFileRoute("/_authenticated/_workspace/documents/")({
  component: DocumentManagementPage,
});
