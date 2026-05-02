import { Navigate, Outlet } from "@tanstack/react-router";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

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
  const [activeWsId, setActiveWsId] = useState(getCurrentWorkspaceId());

  console.log("[WorkspaceGuard] State:", {
    workspacesCount: workspaces.length,
    isLoading,
    isError,
    activeWsId,
  });

  useEffect(() => {
    // Nếu có workspace nhưng chưa có active ID trong localStorage
    if (!isLoading && workspaces.length > 0 && !activeWsId) {
      console.log(
        "[WorkspaceGuard] Auto-setting first workspace:",
        workspaces[0].id,
      );
      setCurrentWorkspaceId(workspaces[0].id);
      // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
      setActiveWsId(workspaces[0].id);
    }
  }, [workspaces, activeWsId, isLoading]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <p className="text-slate-500 dark:text-slate-400 text-sm">
            {t("common.loading_workspace")}
          </p>
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
        <div className="text-center p-8 bg-white dark:bg-slate-900 rounded-3xl shadow-xl border border-slate-100 dark:border-slate-800 max-w-sm">
          <p className="text-red-500 font-bold mb-4">{t("common.system_connection_error")}</p>
          <button
            className="px-6 py-2 bg-primary-600 text-white rounded-xl font-bold"
            onClick={() => { window.location.reload(); }}
          >
            {t("common.retry")}
          </button>
        </div>
      </div>
    );
  }

  if (workspaces.length === 0) {
    return <Navigate replace to={ROUTES.ONBOARDING} />;
  }

  // Nếu chưa có activeWsId nhưng có workspaces, ta render Outlet kèm theo giá trị vừa set
  // Điều này tránh được việc reload trang
  const currentWsId =
    activeWsId || (workspaces.length > 0 ? workspaces[0].id : null);

  if (!currentWsId) {
    return <Navigate replace to={ROUTES.ONBOARDING} />;
  }

  return <Outlet />;
}
