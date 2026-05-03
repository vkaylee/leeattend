import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import React from "react";

import { profileKeys } from "@/hooks/use-profile";
import { fetchMe, clearAuthSession } from "@/lib/api";

function AuthPendingComponent() {
  const { t } = (window as unknown as { __i18n_t__?: { t: (k: string) => string } }).__i18n_t__ ?? { t: (k: string) => k };
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
      <div className="flex flex-col items-center gap-4">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        <p className="text-slate-500 dark:text-slate-400 text-sm">
          {t("auth.checking_security")}
        </p>
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
      });
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      if (!user) {
        clearAuthSession();
        // eslint-disable-next-line @typescript-eslint/only-throw-error
        throw redirect({ to: "/login" });
      }
      return { user };
    } catch (error: unknown) {
      // If it's already a redirect, rethrow
      if (error && typeof error === 'object' && 'isRedirect' in error) throw error;
      
      // Prevent infinite redirect loops by clearing the session 
      // before sending the user back to the login page.
      clearAuthSession();
      
      // Auth failure or network error → redirect to login
      // eslint-disable-next-line @typescript-eslint/only-throw-error
      throw redirect({ to: "/login" });
    }
  },
  pendingComponent: AuthPendingComponent,
  component: Outlet,
});
