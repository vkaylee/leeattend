import { Navigate, Outlet } from "@tanstack/react-router";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useWorkspaces } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId, setCurrentWorkspaceId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";

/**
 * WorkspaceGuard: Đảm bảo người dùng luôn thuộc về ít nhất một Workspace.
 * - Nếu không có workspace nào: Chuyển hướng tới /onboarding.
 * - Nếu có workspace nhưng chưa chọn workspace active: Tự động chọn cái đầu tiên.
 */
export default function WorkspaceGuard() {
  const { t } = useTranslation();
  const { workspaces, isLoading, isError } = useWorkspaces();
  const storedId = getCurrentWorkspaceId();

  // Compute the active workspace ID during render — no useEffect needed.
  // If the stored ID is still valid, keep it; otherwise fall back to the first workspace.
  const activeWsId = useMemo(() => {
    if (!isLoading && workspaces.length > 0) {
      const isValid = storedId && workspaces.some((w) => w.id === storedId);
      if (!storedId || !isValid) {
        setCurrentWorkspaceId(workspaces[0].id);
        return workspaces[0].id;
      }
      return storedId;
    }
    return storedId;
  }, [workspaces, isLoading, storedId]);

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

  if (workspaces.length === 0) {
    return <Navigate replace to={ROUTES.ONBOARDING} />;
  }

  if (!activeWsId) {
    return <Navigate replace to={ROUTES.ONBOARDING} />;
  }

  return <Outlet />;
}
