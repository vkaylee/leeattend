import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspaceDangerPage from "@/pages/workspace/WorkspaceDangerPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/danger')({
  component: WorkspaceDangerPage,
});
