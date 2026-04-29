import React from "react";

import { createFileRoute, redirect } from "@tanstack/react-router";

import MainLayout from "@/components/layout/MainLayout";

import { workspaceKeys } from "@/hooks/use-workspace";
import { fetchWorkspaces, getCurrentWorkspaceId, setCurrentWorkspaceId } from "@/lib/api";

function WorkspacePendingComponent() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
      <div className="flex flex-col items-center gap-4">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        <p className="text-slate-500 dark:text-slate-400 text-sm">Loading workspace...</p>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/_authenticated/_workspace")({
  beforeLoad: async ({ context }) => {
    const workspaces = await context.queryClient.ensureQueryData({
      queryKey: workspaceKeys.all,
      queryFn: fetchWorkspaces,
    });

    if (workspaces.length === 0) {
      throw redirect({ to: "/onboarding" });
    }

    // Auto-set first workspace if none active
    const activeId = getCurrentWorkspaceId();
    if (!activeId && workspaces.length > 0) {
      setCurrentWorkspaceId(workspaces[0].id);
    }

    return { workspaces };
  },
  pendingComponent: WorkspacePendingComponent,
  component: MainLayout,
});
