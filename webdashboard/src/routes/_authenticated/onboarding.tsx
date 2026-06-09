import { createFileRoute, redirect } from "@tanstack/react-router";

import { workspaceKeys } from "@/hooks/use-workspace";
import { fetchWorkspaces } from "@/lib/api";

export const Route = createFileRoute("/_authenticated/onboarding")({
  beforeLoad: async ({ context }) => {
    const workspaces = await context.queryClient.ensureQueryData({
      queryKey: workspaceKeys.all,
      queryFn: fetchWorkspaces,
    });

    if (workspaces.length > 0) {
      // eslint-disable-next-line @typescript-eslint/only-throw-error
      throw redirect({ to: "/" });
    }
  },
});

