import { Outlet } from "@tanstack/react-router";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useWorkspace } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId } from "@/lib/api";

/** Ensures standalone workspace context is loaded before rendering protected routes. */
export default function WorkspaceGuard() {
  const { t } = useTranslation();
  const configuredWorkspaceID = getCurrentWorkspaceId();
  const { workspace, isLoading, isError } = useWorkspace(configuredWorkspaceID ?? undefined);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <Typography className="text-slate-500 dark:text-slate-400 text-sm" variant="p">
            {t("common.loading_workspace")}
          </Typography>
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
        <div className="text-center p-8 bg-white dark:bg-slate-900 rounded-3xl shadow-xl border border-slate-100 dark:border-slate-800 max-w-sm">
          <Typography className="text-red-500 font-bold mb-4" variant="p">{t("common.system_connection_error")}</Typography>
          <Button className="px-6 py-2 bg-primary-600 text-white rounded-xl font-bold"
            onClick={() => { window.location.reload(); }}
            variant="ghost"
          >
            {t("common.retry")}
          </Button>
        </div>
      </div>
    );
  }
  if (!workspace) return null;

  return <Outlet />;
}
