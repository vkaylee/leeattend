import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/")({
  beforeLoad: () => {
    throw redirect({ to: "/workspace/settings/general" });
  },
});
