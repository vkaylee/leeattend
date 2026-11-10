import { createFileRoute } from "@tanstack/react-router";

import { z } from "zod";

const newWorkflowSearchSchema = z.object({
  tab: z.enum(["overtime", "leave", "attendance"]).optional(),
});

export const Route = createFileRoute(
  '/_authenticated/_organization/settings/workflows/new',
)({
  validateSearch: newWorkflowSearchSchema,
});
