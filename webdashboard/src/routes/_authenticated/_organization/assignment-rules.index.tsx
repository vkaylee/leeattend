import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute('/_authenticated/_organization/assignment-rules/')({
  beforeLoad: () => {
    throw redirect({
      to: "/shifts/rules",
      search: { tab: "rules" },
    });
  },
});
