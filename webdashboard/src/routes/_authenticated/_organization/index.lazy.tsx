import { createLazyFileRoute } from "@tanstack/react-router";
import { Suspense, lazy } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Skeleton } from "@/components/ui/skeleton";

// Lazy-load the modern desktop OS page
const DesktopPage = lazy(() => import("@/pages/desktop-page"));

function DesktopFallback() {
  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <div className="flex items-center justify-between">
        <Skeleton className="h-10 w-64 rounded-2xl" />
        <Skeleton className="h-10 w-48 rounded-2xl" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {Array.from({ length: 6 }).map((_, i) => (
          <Skeleton className="h-56 rounded-3xl" key={i} />
        ))}
      </div>
    </div>
  );
}

function IndexPage() {
  return (
    <ErrorBoundary variant="full">
      <Suspense fallback={<DesktopFallback />}>
        <DesktopPage />
      </Suspense>
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_organization/")({
  component: IndexPage,
});
