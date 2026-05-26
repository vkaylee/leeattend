import { QueryClient } from "@tanstack/react-query";
import { createRouter } from "@tanstack/react-router";

import { GlobalPageSkeleton } from "./components/common/global-page-skeleton";
import { RouteErrorFallback } from "./components/common/route-error-fallback";
import { routeTree } from "./route-tree.gen";

export function createAppRouter(queryClient: QueryClient) {
  return createRouter({
    routeTree,
    context: { queryClient },
    defaultPreload: "intent",
    defaultPreloadStaleTime: 0,
    scrollRestoration: true,
    defaultPendingComponent: GlobalPageSkeleton,
    defaultErrorComponent: RouteErrorFallback,
  });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
