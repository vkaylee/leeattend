import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspaceOverviewPage from "@/pages/workspace/WorkspaceOverviewPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/overview')({
  component: WorkspaceOverviewPage,
});
