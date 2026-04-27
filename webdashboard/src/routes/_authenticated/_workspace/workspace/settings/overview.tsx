import { createFileRoute } from "@tanstack/react-router";
import WorkspaceOverviewPage from "@/pages/workspace/WorkspaceOverviewPage";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/overview"
)({
  component: WorkspaceOverviewPage,
});
