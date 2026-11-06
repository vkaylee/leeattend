import { createLazyFileRoute } from "@tanstack/react-router";
import { Suspense, lazy } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Skeleton } from "@/components/ui/skeleton";

const DesktopPage = lazy(() => import("@/pages/desktop-page"));

function DashboardPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <Suspense fallback={<Skeleton className="h-96 w-full rounded-3xl" />}>
        <DesktopPage />
      </Suspense>
    </ErrorBoundary>
  );
}
export const Route = createLazyFileRoute("/_authenticated/_workspace/dashboard")({
  component: DashboardPageWithErrorBoundary,
});
