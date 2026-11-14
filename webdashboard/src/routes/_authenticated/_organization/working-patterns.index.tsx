import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute('/_authenticated/_organization/working-patterns/')({
  beforeLoad: () => {
    throw redirect({
      to: "/shifts/rules",
      search: { tab: "patterns" },
    });
  },
});
