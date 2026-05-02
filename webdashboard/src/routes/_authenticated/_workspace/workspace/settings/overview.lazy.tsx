import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspaceOverviewPage from "@/pages/workspace/workspace-overview-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/overview')({
  component: WorkspaceOverviewPage,
});
