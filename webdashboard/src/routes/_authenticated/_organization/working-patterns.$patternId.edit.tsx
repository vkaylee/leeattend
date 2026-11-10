import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute('/_authenticated/_organization/working-patterns/$patternId/edit')({
  beforeLoad: ({ params }) => {
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw redirect({
      to: "/shifts/rules/working-patterns/$patternId/edit",
      params: { patternId: params.patternId },
    });
  },
});
