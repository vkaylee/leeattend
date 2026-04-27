import { createFileRoute } from "@tanstack/react-router";
import WorkspaceRolesPage from "@/pages/workspace/WorkspaceRolesPage";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/roles"
)({
  component: WorkspaceRolesPage,
});
