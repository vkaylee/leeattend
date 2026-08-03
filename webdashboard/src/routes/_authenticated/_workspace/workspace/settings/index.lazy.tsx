import { createLazyFileRoute } from "@tanstack/react-router";
import { Suspense, lazy } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Skeleton } from "@/components/ui/skeleton";

const SettingsHubPage = lazy(() => import("@/pages/settings/settings-hub-page"));

function SettingsHubFallback() {
  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        {Array.from({ length: 8 }).map((_, i) => (
          <Skeleton className="h-28 rounded-2xl" key={i} />
        ))}
      </div>
    </div>
  );
}

export const Route = createLazyFileRoute("/_authenticated/_workspace/workspace/settings/")({
  component: () => (
    <ErrorBoundary variant="full">
      <Suspense fallback={<SettingsHubFallback />}>
        <SettingsHubPage />
      </Suspense>
    </ErrorBoundary>
  ),
});
