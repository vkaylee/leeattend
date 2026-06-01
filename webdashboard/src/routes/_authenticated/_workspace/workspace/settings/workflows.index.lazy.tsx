import { createLazyFileRoute } from "@tanstack/react-router";

import WorkflowsSettings from "@/pages/settings/workflows-settings";

export const Route = createLazyFileRoute("/_authenticated/_workspace/workspace/settings/workflows/")({
  component: WorkflowsSettings,
});
