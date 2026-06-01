import { createLazyFileRoute } from "@tanstack/react-router";

import WorkflowEditorPage from "@/pages/settings/workflow-editor-page";

export const Route = createLazyFileRoute(
  '/_authenticated/_workspace/workspace/settings/workflows/$workflowId/edit',
)({
  component: WorkflowEditorPage,
});
