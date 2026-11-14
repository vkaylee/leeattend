import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute('/_authenticated/_organization/assignment-rules/new')({
  beforeLoad: () => {
    throw redirect({
      to: "/shifts/rules/assignment-rules/new",
    });
  },
});
