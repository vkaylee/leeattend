import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { profileKeys } from "@/hooks/use-profile";
import { fetchMe, clearAuthSession } from "@/lib/api";
import { maybeNull } from "@/types/api-overrides";

function AuthPendingComponent() {
  const { t } = useTranslation();
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
      <div className="flex flex-col items-center gap-4">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        <Typography className="text-slate-500 dark:text-slate-400 text-sm" variant="p">
          {t("auth.checking_security")}
        </Typography>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/_authenticated")({
  beforeLoad: async ({ context }) => {
    try {
      const user = await context.queryClient.ensureQueryData({
        queryKey: profileKeys.me,
        queryFn: fetchMe,
        retry: false,
      });
      if (maybeNull(user) === null || maybeNull(user) === undefined) {
        clearAuthSession();
        // eslint-disable-next-line @typescript-eslint/only-throw-error
        throw redirect({ to: "/login" });
      }
      return { user };
    } catch (error: unknown) {
      // If it's already a redirect, rethrow.
      if (error && typeof error === 'object' && 'isRedirect' in error) throw error;

      // apiFetch already clears memory and expires cookies when refresh fails.
      // Avoid a second logout request during the same auth bootstrap failure.
      if (!(error instanceof Error && error.message === "Session expired")) {
        clearAuthSession();
      }

      // Auth failure or network error → redirect to login
      // eslint-disable-next-line @typescript-eslint/only-throw-error
      throw redirect({ to: "/login" });
    }
  },
  pendingComponent: AuthPendingComponent,
  component: Outlet,
});
