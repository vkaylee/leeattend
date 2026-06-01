import { createFileRoute } from "@tanstack/react-router";

import { z } from "zod";

const editWorkflowSearchSchema = z.object({
  tab: z.enum(["overtime", "leave", "attendance"]).optional(),
});

export const Route = createFileRoute(
  '/_authenticated/_workspace/workspace/settings/workflows/$workflowId/edit',
)({
  validateSearch: editWorkflowSearchSchema,
});
