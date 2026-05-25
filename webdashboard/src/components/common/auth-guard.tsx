import { Navigate, Outlet, useLocation } from "@tanstack/react-router";
import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useProfile } from "@/hooks/use-profile";
import { getCurrentUserId, clearAuthSession } from "@/lib/api";
import { ROUTES } from "@/lib/routes";

/**
 * AuthGuard: Bảo vệ các routes yêu cầu đăng nhập.
 * Nếu chưa có userId hoặc userId không hợp lệ (không tồn tại trong DB),
 * chuyển hướng về /login.
 */
export default function AuthGuard() {
  const { t } = useTranslation();
  const _location = useLocation();
  const _userId = getCurrentUserId();
  const { user, isLoading, isError } = useProfile();

  

  useEffect(() => {
    if (isError) {
      
      clearAuthSession();
    }
  }, [isError]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <Typography className="text-slate-500 dark:text-slate-400 text-sm" variant="p">{t("auth.checking_security")}</Typography>
        </div>
      </div>
    );
  }

  if (isError || !user) {
    // Chuyển hướng sang login kèm theo state "from" để quay lại sau
    return <Navigate replace to={ROUTES.LOGIN} />;
  }

  return <Outlet />;
}

