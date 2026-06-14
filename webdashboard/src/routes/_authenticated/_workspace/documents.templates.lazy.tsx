import { createLazyFileRoute } from "@tanstack/react-router";
import { DocumentTemplatesPage } from "@/pages/documents/DocumentTemplatesPage";

export const Route = createLazyFileRoute("/_authenticated/_workspace/documents/templates")({
  component: DocumentTemplatesPage,
});
