import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/"
)({
  beforeLoad: () => {
    // eslint-disable-next-line @typescript-eslint/only-throw-error -- TanStack Router redirect pattern
    throw redirect({
      to: "/workspace/settings/general",
    });
  },
});

