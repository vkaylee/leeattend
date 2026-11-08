import type { QueryClient } from "@tanstack/react-query";

import { createRootRouteWithContext, Outlet, useRouterState } from "@tanstack/react-router";
import { lazy, Suspense } from "react";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Typography } from "@/components/ui/typography";

const TanStackRouterDevtools = import.meta.env.DEV
  ? lazy(() =>
      import("@tanstack/router-devtools").then((res) => ({
        default: res.TanStackRouterDevtools,
      }))
    )
  : () => null;


export interface RouterContext {
  queryClient: QueryClient;
}

function NotFoundPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
      <div className="text-center space-y-4">
        <Typography className="text-6xl font-bold text-slate-300 dark:text-slate-600" variant="h1">404</Typography>
        <Typography className="text-lg text-slate-500 dark:text-slate-400" variant="p">Page not found</Typography>
        <a
          className="inline-block px-6 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
          href="/"
        >
          Back to home
        </a>
      </div>
    </div>
  );
}

function RootComponent() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const showDevtools = import.meta.env.DEV && !["/login", "/register", "/forgot-password", "/reset-password"].includes(pathname);

  return (
    <ErrorBoundary>
      <Outlet />
      {showDevtools ? (
        <Suspense fallback={null}>
          <TanStackRouterDevtools position="bottom-right" />
        </Suspense>
      ) : null}
    </ErrorBoundary>
  );
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  notFoundComponent: NotFoundPage,
});
