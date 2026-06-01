import { createFileRoute, Outlet } from "@tanstack/react-router";

import { z } from "zod";

const workflowsSearchSchema = z.object({
  tab: z.enum(["overtime", "leave", "attendance"]).optional(),
});

export const Route = createFileRoute(
  '/_authenticated/_workspace/workspace/settings/workflows',
)({
  validateSearch: workflowsSearchSchema,
  component: Outlet,
});


