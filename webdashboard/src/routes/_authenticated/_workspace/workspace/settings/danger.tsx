import { createFileRoute } from "@tanstack/react-router";
import WorkspaceDangerPage from "@/pages/workspace/WorkspaceDangerPage";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/danger"
)({
  component: WorkspaceDangerPage,
});
