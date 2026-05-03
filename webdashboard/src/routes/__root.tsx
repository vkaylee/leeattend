import type { QueryClient } from "@tanstack/react-query";

import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";

import { TanStackRouterDevtools } from "@tanstack/router-devtools";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { DebugToolbar } from "@/components/debug/debug-toolbar";

export interface RouterContext {
  queryClient: QueryClient;
}

function NotFoundPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-800">
      <div className="text-center space-y-4">
        <h1 className="text-6xl font-bold text-slate-300 dark:text-slate-600">404</h1>
        <p className="text-lg text-slate-500 dark:text-slate-400">Page not found</p>
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

export const Route = createRootRouteWithContext<RouterContext>()({
  component: () => (
    <ErrorBoundary>
      <Outlet />
      <DebugToolbar />
      {import.meta.env.DEV ? <TanStackRouterDevtools /> : null}
    </ErrorBoundary>
  ),
  notFoundComponent: NotFoundPage,
});
