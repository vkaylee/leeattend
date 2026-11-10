import { createLazyFileRoute } from "@tanstack/react-router";
import { Suspense, lazy } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Skeleton } from "@/components/ui/skeleton";

const NotificationsPage = lazy(() => import("@/pages/notifications-page"));

function NotificationsFallback() {
  return (
    <div className="animate-in fade-in duration-500 space-y-6 pb-20">
      {/* Header skeleton */}
      <div className="flex items-center gap-4 mb-2">
        <Skeleton className="h-10 w-10 rounded-full" />
        <div className="space-y-2">
          <Skeleton className="h-6 w-48 rounded-lg" />
          <Skeleton className="h-4 w-64 rounded-lg" />
        </div>
      </div>
      {/* Content skeleton */}
      <div className="max-w-4xl mx-auto space-y-4">
        {Array.from({ length: 5 }).map((_, i) => (
          <div className="bg-white dark:bg-slate-800 rounded-2xl border border-slate-100 dark:border-slate-700 p-5 flex gap-4" key={i}>
            <Skeleton className="h-10 w-10 rounded-full shrink-0" />
            <div className="flex-1 space-y-2">
              <Skeleton className="h-4 w-3/4 rounded-lg" />
              <Skeleton className="h-3 w-full rounded-lg" />
              <Skeleton className="h-3 w-1/2 rounded-lg" />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/notifications")({
  component: () => (
    <ErrorBoundary variant="full">
      <Suspense fallback={<NotificationsFallback />}>
        <NotificationsPage />
      </Suspense>
    </ErrorBoundary>
  ),
});
