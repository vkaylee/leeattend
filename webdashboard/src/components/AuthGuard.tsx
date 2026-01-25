import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Navigate, Outlet } from "react-router-dom";
import { getCurrentUserId, clearAuthSession } from "@/lib/api";
import { useProfile } from "@/hooks/use-profile";

/**
 * AuthGuard: Bảo vệ các routes yêu cầu đăng nhập.
 * Nếu chưa có userId hoặc userId không hợp lệ (không tồn tại trong DB),
 * chuyển hướng về /login.
 */
export default function AuthGuard() {
  const { t } = useTranslation();
  const userId = getCurrentUserId();
  const { user, isLoading, isError } = useProfile();

  console.log("[AuthGuard] State:", {
    userId,
    hasUser: !!user,
    isLoading,
    isError,
  });

  useEffect(() => {
    if (isError) {
      console.log("[AuthGuard] Error detected, redirecting to login...");
      clearAuthSession();
      window.location.href = "/login";
    }
  }, [isError]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <p className="text-slate-500 text-sm">{t("auth.checking_security")}</p>
        </div>
      </div>
    );
  }

  if (isError || !user) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <p className="text-slate-500 text-sm animate-pulse">
            {isError
              ? t("auth.session_expired")
              : t("auth.user_not_found")}
          </p>
        </div>
      </div>
    );
  }

  return <Outlet />;
}
