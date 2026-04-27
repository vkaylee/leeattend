import { createFileRoute } from "@tanstack/react-router";
import WorkspacePermissionsPage from "@/pages/workspace/WorkspacePermissionsPage";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/permissions"
)({
  component: WorkspacePermissionsPage,
});
