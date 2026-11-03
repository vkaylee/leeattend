import { createFileRoute } from "@tanstack/react-router";
import React from "react";

import AppShell from "@/components/layout/app-shell";
import { Typography } from "@/components/ui/typography";
import { fetchWorkspace, getCurrentWorkspaceId } from "@/lib/api";

function WorkspacePendingComponent() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
      <div className="flex flex-col items-center gap-4">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        <Typography className="text-slate-500 dark:text-slate-400 text-sm" variant="p">Loading workspace...</Typography>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/_authenticated/_workspace")({
  beforeLoad: async () => {
    const workspaceID = getCurrentWorkspaceId();
    if (!workspaceID) return {};
    return { workspace: await fetchWorkspace(workspaceID) };
  },
  pendingComponent: WorkspacePendingComponent,
  component: AppShell,
});
