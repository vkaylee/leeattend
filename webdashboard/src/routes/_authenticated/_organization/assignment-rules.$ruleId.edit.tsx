import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute('/_authenticated/_organization/assignment-rules/$ruleId/edit')({
  beforeLoad: ({ params }) => {
    throw redirect({
      to: "/shifts/rules/assignment-rules/$ruleId/edit",
      params: { ruleId: params.ruleId },
    });
  },
});
