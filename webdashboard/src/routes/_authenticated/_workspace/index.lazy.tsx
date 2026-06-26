import { createLazyFileRoute } from "@tanstack/react-router";
import React, { Suspense } from "react";

import { Skeleton } from "@/components/ui/skeleton";
import DashboardPage from "@/pages/dashboard-page";

// Lazy-load the app launcher to keep the main bundle lean
const AppLauncherPage = React.lazy(() => import("@/pages/app-launcher-page"));

function AppLauncherFallback() {
  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
        {Array.from({ length: 5 }).map((_, i) => (
          <div className="flex items-center gap-3 p-4 rounded-2xl bg-card border border-slate-100 dark:border-slate-800" key={i}>
            <Skeleton className="w-10 h-10 rounded-xl shrink-0" />
            <div className="space-y-1.5 flex-1">
              <Skeleton className="h-3 w-16" />
              <Skeleton className="h-5 w-12" />
            </div>
          </div>
        ))}
      </div>
      <div>
        <Skeleton className="h-5 w-32 mb-4" />
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {Array.from({ length: 8 }).map((_, i) => (
            <Skeleton className="h-32 rounded-2xl" key={i} />
          ))}
        </div>
      </div>
    </div>
  );
}

function IndexPage() {
  // Backward-compatible: set localStorage "leeattend-legacy-dashboard" to "true" for old dashboard
  const useLegacyDashboard = (() => {
    try {
      return localStorage.getItem("leeattend-legacy-dashboard") === "true";
    } catch {
      return false;
    }
  })();

  if (useLegacyDashboard) {
    return <DashboardPage />;
  }

  return (
    <Suspense fallback={<AppLauncherFallback />}>
      <AppLauncherPage />
    </Suspense>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_workspace/")({
  component: IndexPage,
});
