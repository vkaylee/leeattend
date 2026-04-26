import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import type { QueryClient } from "@tanstack/react-query";
import { DebugToolbar } from "@/components/debug/DebugToolbar";

export interface RouterContext {
  queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: () => (
    <>
      <Outlet />
      <DebugToolbar />
      {import.meta.env.DEV && <TanStackRouterDevtools />}
    </>
  ),
});
