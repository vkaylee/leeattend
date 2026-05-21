import { createLazyFileRoute } from "@tanstack/react-router";

import WorkingPatternsEditorPage from "@/pages/working-patterns-editor-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/working-patterns/new')({
  component: WorkingPatternsEditorPage,
});
