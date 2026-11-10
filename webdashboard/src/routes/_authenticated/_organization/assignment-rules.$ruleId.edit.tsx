import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute('/_authenticated/_organization/assignment-rules/$ruleId/edit')({
  beforeLoad: ({ params }) => {
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw redirect({
      to: "/shifts/rules/assignment-rules/$ruleId/edit",
      params: { ruleId: params.ruleId },
    });
  },
});
