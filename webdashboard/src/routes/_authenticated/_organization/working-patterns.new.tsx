import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute('/_authenticated/_organization/working-patterns/new')({
  beforeLoad: () => {
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw redirect({
      to: "/shifts/rules/working-patterns/new",
    });
  },
});
