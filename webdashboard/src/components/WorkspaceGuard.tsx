import React, { useEffect } from "react";
import { Navigate, Outlet } from "react-router-dom";
import { useWorkspaces } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId, setCurrentWorkspaceId } from "@/lib/api";

/**
 * WorkspaceGuard: Đảm bảo người dùng luôn thuộc về ít nhất một Workspace.
 * - Nếu không có workspace nào: Chuyển hướng tới /onboarding.
 * - Nếu có workspace nhưng chưa chọn workspace active: Tự động chọn cái đầu tiên.
 */
export default function WorkspaceGuard() {
  const { workspaces, isLoading, isError } = useWorkspaces();
  const activeWsId = getCurrentWorkspaceId();

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
    }
  }, [workspaces, activeWsId, isLoading]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <p className="text-slate-400 text-sm">
            Đang tải không gian làm việc...
          </p>
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50">
        <div className="text-center p-8 bg-white rounded-3xl shadow-xl border border-slate-100 max-w-sm">
          <p className="text-red-500 font-bold mb-4">Lỗi kết nối hệ thống</p>
          <button
            onClick={() => window.location.reload()}
            className="px-6 py-2 bg-primary-600 text-white rounded-xl font-bold"
          >
            Thử lại
          </button>
        </div>
      </div>
    );
  }

  if (workspaces.length === 0) {
    return <Navigate to="/onboarding" replace />;
  }

  // Nếu chưa có activeWsId nhưng có workspaces, ta render Outlet kèm theo giá trị vừa set
  // Điều này tránh được việc reload trang
  const currentWsId =
    activeWsId || (workspaces.length > 0 ? workspaces[0].id : null);

  if (!currentWsId) {
    return <Navigate to="/onboarding" replace />;
  }

  return <Outlet />;
}
